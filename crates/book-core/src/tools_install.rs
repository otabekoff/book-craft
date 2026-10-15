//! Downloads missing packaging tools into a folder Book Craft searches first, so
//! building setup files, MSIs and MSIX packages works without installing anything
//! system-wide (no administrator rights needed).
//!
//! - NSIS 3 and WiX 3.14: the same archives Tauri uses for its own installers.
//! - MakeAppx and SignTool: Microsoft's `Microsoft.Windows.SDK.BuildTools` NuGet package.
//! - WebView2 bootstrapper: Microsoft's evergreen bootstrapper.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, bail};
use serde::Serialize;

/// A tool Book Craft can fetch, with the winget command to install it system-wide instead.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
    pub winget: &'static str,
}

pub const DOWNLOADS: &[Download] = &[
    Download {
        id: "nsis",
        name: "NSIS",
        url: "https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.13/nsis-3.13.zip",
        winget: "winget install --id NSIS.NSIS -e",
    },
    Download {
        id: "wix",
        name: "WiX Toolset 3",
        url: "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip",
        winget: "winget install --id WiXToolset.WiXToolset -e",
    },
    Download {
        id: "sdk",
        name: "MakeAppx and SignTool",
        // Resolved to the newest stable version when downloading.
        url: "nuget:microsoft.windows.sdk.buildtools",
        winget: "winget install --id Microsoft.WindowsSDK.10.0.26100 -e",
    },
    Download {
        id: "webview2",
        name: "WebView2 bootstrapper",
        url: "https://go.microsoft.com/fwlink/p/?LinkId=2124703",
        winget: "",
    },
];

/// Where downloaded tools go: `%LOCALAPPDATA%/uz.nurafshon.bookcraft/tools`, shared by
/// the studio and the `bookcraft` command line.
pub fn default_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("uz.nurafshon.bookcraft").join("tools"))
}

/// The download that provides a tool shown in [`crate::tools::ToolPaths::status`].
pub fn download_for(tool_name: &str) -> Option<&'static Download> {
    let id = match tool_name {
        "NSIS" => "nsis",
        "WiX Toolset 3" => "wix",
        "MakeAppx" | "SignTool" => "sdk",
        "WebView2 bootstrapper" => "webview2",
        _ => return None,
    };
    DOWNLOADS.iter().find(|d| d.id == id)
}

/// Downloads and unpacks one tool into `tools_dir`. Returns the folder or file it created.
pub fn install(id: &str, tools_dir: &Path, log: &mut dyn FnMut(&str)) -> anyhow::Result<PathBuf> {
    let d = DOWNLOADS.iter().find(|d| d.id == id).with_context(|| format!("Unknown tool \"{id}\""))?;
    std::fs::create_dir_all(tools_dir).with_context(|| format!("Creating {}", tools_dir.display()))?;
    let tmp = tools_dir.join(format!(".download-{id}.tmp"));
    log(&format!("Downloading {}…", d.name));
    fetch(d.url, &tmp)?;
    let result = (|| match id {
        "nsis" => unpack(&tmp, tools_dir, "NSIS", |p| Some(p.to_path_buf()), "makensis.exe"),
        "wix" => unpack(&tmp, tools_dir, "WixTools314", |p| Some(p.to_path_buf()), "candle.exe"),
        // Only the 64-bit tools: bin/<version>/x64/* → WindowsSDK/x64/*
        "sdk" => unpack(
            &tmp,
            tools_dir,
            "WindowsSDK",
            |p| {
                let parts: Vec<_> = p.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
                (parts.len() >= 4 && parts[0] == "bin" && parts[2] == "x64").then(|| PathBuf::from("x64").join(parts[3..].join("/")))
            },
            "x64/makeappx.exe",
        ),
        "webview2" => {
            let dest = tools_dir.join("MicrosoftEdgeWebview2Setup.exe");
            std::fs::copy(&tmp, &dest).with_context(|| format!("Saving {}", dest.display()))?;
            Ok(dest)
        }
        _ => unreachable!(),
    })();
    let _ = std::fs::remove_file(&tmp);
    let path = result?;
    log(&format!("{} is ready: {}", d.name, path.display()));
    Ok(path)
}

/// A client that gives up on a dead connection instead of waiting forever.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_recv_response(Some(Duration::from_secs(60)))
        .user_agent(concat!("BookCraft/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

/// `nuget:<id>` → the newest stable package from NuGet's flat container.
fn resolve(url: &str) -> anyhow::Result<String> {
    let Some(id) = url.strip_prefix("nuget:") else { return Ok(url.to_string()) };
    let index = format!("https://api.nuget.org/v3-flatcontainer/{id}/index.json");
    let text = agent()
        .get(&index)
        .call()
        .and_then(|r| r.into_body().read_to_string())
        .with_context(|| format!("Looking up {id} on NuGet"))?;
    let versions: serde_json::Value = serde_json::from_str(&text)?;
    let version = versions["versions"]
        .as_array()
        .and_then(|v| v.iter().filter_map(|x| x.as_str()).filter(|x| !x.contains('-')).last())
        .with_context(|| format!("NuGet listed no stable version of {id}"))?;
    Ok(format!("https://api.nuget.org/v3-flatcontainer/{id}/{version}/{id}.{version}.nupkg"))
}

fn fetch(url: &str, dest: &Path) -> anyhow::Result<()> {
    let url = resolve(url)?;
    let url = url.as_str();
    let response = agent().get(url).call().with_context(|| format!("Downloading {url}"))?;
    let mut body = response.into_body().into_reader();
    let mut file = File::create(dest).with_context(|| format!("Creating {}", dest.display()))?;
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = body.read(&mut buf).with_context(|| format!("Downloading {url}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
    }
    file.flush()?;
    Ok(())
}

/// Extracts `archive` into `tools_dir/<folder>`, mapping each entry's path with `map`
/// (entries mapped to `None` are skipped). A single top-level folder shared by every
/// entry is dropped. The folder is replaced only once extraction succeeded and
/// `check` (a file inside it) exists.
fn unpack(archive: &Path, tools_dir: &Path, folder: &str, map: impl Fn(&Path) -> Option<PathBuf>, check: &str) -> anyhow::Result<PathBuf> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).context("The download is not a valid archive")?;
    let names: Vec<PathBuf> = (0..zip.len()).filter_map(|i| zip.by_index(i).ok().and_then(|e| e.enclosed_name())).collect();
    let first = names.first().and_then(|n| n.components().next()).map(|c| c.as_os_str().to_owned());
    let strip = first.filter(|f| names.len() > 1 && names.iter().all(|n| n.components().count() > 1 && n.components().next().map(|c| c.as_os_str()) == Some(f.as_os_str())));

    let partial = tools_dir.join(format!("{folder}.partial"));
    let _ = std::fs::remove_dir_all(&partial);
    std::fs::create_dir_all(&partial)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(name) = entry.enclosed_name() else { continue };
        let name = match &strip {
            Some(s) => name.strip_prefix(s).map(Path::to_path_buf).unwrap_or(name),
            None => name,
        };
        let Some(rel) = map(&name) else { continue };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let out = partial.join(&rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut f = File::create(&out).with_context(|| format!("Writing {}", out.display()))?;
        std::io::copy(&mut entry, &mut f)?;
    }
    if !partial.join(check).is_file() {
        let _ = std::fs::remove_dir_all(&partial);
        bail!("The download did not contain {check}");
    }
    let dest = tools_dir.join(folder);
    if dest.exists() {
        // Keep the old copy aside rather than deleting it, in case it is in use.
        let old = tools_dir.join(format!("{folder}.old-{}", std::process::id()));
        std::fs::rename(&dest, &old).with_context(|| format!("Replacing {}", dest.display()))?;
        let _ = std::fs::remove_dir_all(&old);
    }
    std::fs::rename(&partial, &dest).with_context(|| format!("Moving {} into place", dest.display()))?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_missing_tool_has_a_download() {
        for name in ["NSIS", "WiX Toolset 3", "MakeAppx", "SignTool", "WebView2 bootstrapper"] {
            assert!(download_for(name).is_some(), "{name}");
        }
    }

    #[test]
    fn unpack_drops_the_shared_top_folder() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.zip");
        {
            let mut w = zip::ZipWriter::new(File::create(&archive).unwrap());
            let opts = zip::write::SimpleFileOptions::default();
            w.start_file("nsis-3.13/makensis.exe", opts).unwrap();
            w.write_all(b"x").unwrap();
            w.start_file("nsis-3.13/Plugins/a.dll", opts).unwrap();
            w.write_all(b"y").unwrap();
            w.finish().unwrap();
        }
        let out = unpack(&archive, dir.path(), "NSIS", |p| Some(p.to_path_buf()), "makensis.exe").unwrap();
        assert!(out.join("makensis.exe").is_file());
        assert!(out.join("Plugins/a.dll").is_file());
    }
}
