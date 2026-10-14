//! Finds the packaging tools on this machine: NSIS, WiX 3, MakeAppx, SignTool
//! and the WebView2 bootstrapper. Every path can be overridden.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ToolPaths {
    pub makensis: String,
    /// Folder that contains candle.exe and light.exe.
    pub wix: String,
    pub makeappx: String,
    pub signtool: String,
    pub webview2_bootstrapper: String,
    /// Folders searched before the standard locations (e.g. the app's bundled tools).
    #[serde(skip)]
    pub extra_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub name: String,
    pub needed_for: String,
    pub path: Option<String>,
    pub help: String,
}

fn existing(p: PathBuf) -> Option<PathBuf> {
    p.is_file().then_some(p)
}

fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var_os(var).map(PathBuf::from)
}

fn on_path(exe: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths).find_map(|d| existing(d.join(exe)))
}

fn tauri_cache() -> Option<PathBuf> {
    env_path("LOCALAPPDATA").map(|p| p.join("tauri"))
}

impl ToolPaths {
    fn override_path(&self, value: &str) -> Option<PathBuf> {
        let v = value.trim();
        if v.is_empty() { None } else { existing(PathBuf::from(v)) }
    }

    fn in_extra(&self, rel: &str) -> Option<PathBuf> {
        self.extra_dirs.iter().find_map(|d| existing(d.join(rel)))
    }

    pub fn makensis(&self) -> Option<PathBuf> {
        self.override_path(&self.makensis)
            .or_else(|| self.in_extra("NSIS/makensis.exe"))
            .or_else(|| tauri_cache().and_then(|t| existing(t.join("NSIS/makensis.exe"))))
            .or_else(|| env_path("ProgramFiles(x86)").and_then(|p| existing(p.join("NSIS/makensis.exe"))))
            .or_else(|| env_path("ProgramFiles").and_then(|p| existing(p.join("NSIS/makensis.exe"))))
            .or_else(|| on_path("makensis.exe"))
    }

    /// Returns the folder holding `candle.exe` and `light.exe`.
    pub fn wix(&self) -> Option<PathBuf> {
        let ok = |d: PathBuf| (d.join("candle.exe").is_file() && d.join("light.exe").is_file()).then_some(d);
        let v = self.wix.trim();
        if !v.is_empty() {
            if let Some(d) = ok(PathBuf::from(v)) {
                return Some(d);
            }
        }
        self.extra_dirs
            .iter()
            .find_map(|d| ok(d.join("WixTools314")))
            .or_else(|| tauri_cache().and_then(|t| ok(t.join("WixTools314"))))
            .or_else(|| env_path("WIX").and_then(|w| ok(w.join("bin"))))
            .or_else(|| {
                env_path("ProgramFiles(x86)")
                    .and_then(|p| ok(p.join("WiX Toolset v3.14/bin")).or_else(|| ok(p.join("WiX Toolset v3.11/bin"))))
            })
            .or_else(|| on_path("candle.exe").and_then(|c| c.parent().map(Path::to_path_buf)).and_then(ok))
    }

    pub fn makeappx(&self) -> Option<PathBuf> {
        self.override_path(&self.makeappx).or_else(|| windows_sdk_tool("makeappx.exe")).or_else(|| on_path("makeappx.exe"))
    }

    pub fn signtool(&self) -> Option<PathBuf> {
        self.override_path(&self.signtool).or_else(|| windows_sdk_tool("signtool.exe")).or_else(|| on_path("signtool.exe"))
    }

    pub fn webview2_bootstrapper(&self) -> Option<PathBuf> {
        self.override_path(&self.webview2_bootstrapper)
            .or_else(|| self.in_extra("MicrosoftEdgeWebview2Setup.exe"))
            .or_else(|| tauri_cache().and_then(|t| existing(t.join("MicrosoftEdgeWebview2Setup.exe"))))
    }

    pub fn status(&self) -> Vec<ToolStatus> {
        let s = |p: Option<PathBuf>| p.map(|p| p.display().to_string());
        vec![
            ToolStatus {
                name: "NSIS".into(),
                needed_for: "Setup .exe".into(),
                path: s(self.makensis()),
                help: "Install NSIS 3 from nsis.sourceforge.io, or point to makensis.exe.".into(),
            },
            ToolStatus {
                name: "WiX Toolset 3".into(),
                needed_for: ".msi".into(),
                path: s(self.wix()),
                help: "Install WiX Toolset 3.14 from wixtoolset.org, or point to its bin folder.".into(),
            },
            ToolStatus {
                name: "MakeAppx".into(),
                needed_for: ".msix".into(),
                path: s(self.makeappx()),
                help: "Install the Windows 10/11 SDK (it includes makeappx.exe).".into(),
            },
            ToolStatus {
                name: "SignTool".into(),
                needed_for: "Code signing".into(),
                path: s(self.signtool()),
                help: "Install the Windows 10/11 SDK (it includes signtool.exe).".into(),
            },
            ToolStatus {
                name: "WebView2 bootstrapper".into(),
                needed_for: "Installing WebView2 on older PCs".into(),
                path: s(self.webview2_bootstrapper()),
                help: "Download MicrosoftEdgeWebview2Setup.exe from Microsoft, or choose \"download\" in Build settings.".into(),
            },
        ]
    }
}

/// Newest `x64` tool in `Windows Kits\10\bin\<version>\x64`.
fn windows_sdk_tool(exe: &str) -> Option<PathBuf> {
    let roots: Vec<PathBuf> = ["ProgramFiles(x86)", "ProgramFiles"]
        .iter()
        .filter_map(|v| env_path(v))
        .map(|p| p.join("Windows Kits/10/bin"))
        .collect();
    let mut best: Option<(Vec<u32>, PathBuf)> = None;
    for root in roots {
        let Ok(entries) = std::fs::read_dir(&root) else { continue };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let ver: Vec<u32> = name.split('.').filter_map(|p| p.parse().ok()).collect();
            if ver.len() != 4 {
                continue;
            }
            if let Some(p) = existing(e.path().join("x64").join(exe)) {
                if best.as_ref().is_none_or(|(v, _)| ver > *v) {
                    best = Some((ver, p));
                }
            }
        }
        if best.is_none() {
            if let Some(p) = existing(root.join("x64").join(exe)) {
                best = Some((vec![0], p));
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Runs a tool and returns its combined output, or an error that includes it.
pub fn run(cmd: &mut Command, what: &str) -> anyhow::Result<String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().map_err(|e| anyhow::anyhow!("Could not start {what}: {e}"))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if !out.status.success() {
        let tail: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        let tail = tail[tail.len().saturating_sub(25)..].join("\n");
        anyhow::bail!("{what} failed ({}):\n{tail}", out.status);
    }
    Ok(text)
}
