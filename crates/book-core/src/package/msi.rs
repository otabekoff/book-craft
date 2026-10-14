//! `.msi` via WiX Toolset 3 (candle + light).

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::Stage;
use crate::tools;

/// XML attribute text; `$` is doubled so WiX's preprocessor leaves it alone,
/// and square brackets are removed because MSI treats them as properties.
fn x(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
        .replace('$', "$$")
        .replace(['[', ']'], "")
        .replace(['\r', '\n'], " ")
}

pub fn upgrade_code(package_id: &str) -> String {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, format!("bookcraft:{package_id}").as_bytes())
        .to_string()
        .to_uppercase()
}

/// Stable per-book component GUIDs keep upgrades and uninstalls clean.
fn component_guid(package_id: &str, component: &str) -> String {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, format!("bookcraft:{package_id}:{component}").as_bytes())
        .to_string()
        .to_uppercase()
}

pub fn build(stage: &Stage, wix_bin: &Path, out: &Path) -> Result<()> {
    let c = &stage.config;
    let b = &c.build;
    let per_machine = b.install_mode == "perMachine";
    let root = if per_machine { "HKLM" } else { "HKCU" };
    let v = c.version_parts();
    let version = format!("{}.{}.{}", v[0], v[1], v[2]);
    let regkey = format!("Software\\{}", x(&c.meta.package_id));
    let product = x(&c.product_name());
    let publisher = if c.meta.publisher.trim().is_empty() { x(&c.meta.author) } else { x(&c.meta.publisher) };
    let publisher = if publisher.is_empty() { "Unknown publisher".to_string() } else { publisher };
    let folder = x(&stage.product_file_name);

    let mut w = String::new();
    w.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<Wix xmlns=\"http://schemas.microsoft.com/wix/2006/wi\">\n");
    w.push_str(&format!(
        "  <Product Id=\"*\" Name=\"{product}\" Language=\"1033\" Codepage=\"65001\" Version=\"{version}\" Manufacturer=\"{publisher}\" UpgradeCode=\"{}\">\n",
        upgrade_code(&c.meta.package_id)
    ));
    w.push_str(&format!(
        "    <Package InstallerVersion=\"500\" Compressed=\"yes\" InstallScope=\"{}\" Platform=\"x64\" Description=\"{product}\" Manufacturer=\"{publisher}\" />\n",
        if per_machine { "perMachine" } else { "perUser" }
    ));
    w.push_str("    <MajorUpgrade AllowSameVersionUpgrades=\"yes\" DowngradeErrorMessage=\"A newer version of this book is already installed.\" />\n");
    w.push_str("    <MediaTemplate EmbedCab=\"yes\" CompressionLevel=\"high\" />\n");
    w.push_str(&format!("    <Icon Id=\"AppIcon.ico\" SourceFile=\"{}\" />\n", x(&stage.ico.display().to_string())));
    w.push_str("    <Property Id=\"ARPPRODUCTICON\" Value=\"AppIcon.ico\" />\n    <Property Id=\"ARPNOMODIFY\" Value=\"1\" />\n    <Property Id=\"ARPNOREPAIR\" Value=\"1\" />\n");
    if !c.meta.website.trim().is_empty() {
        w.push_str(&format!("    <Property Id=\"ARPURLINFOABOUT\" Value=\"{}\" />\n", x(&c.meta.website)));
    }

    w.push_str("    <Directory Id=\"TARGETDIR\" Name=\"SourceDir\">\n");
    if per_machine {
        w.push_str(&format!("      <Directory Id=\"ProgramFiles64Folder\">\n        <Directory Id=\"INSTALLDIR\" Name=\"{folder}\" />\n      </Directory>\n"));
    } else {
        w.push_str(&format!(
            "      <Directory Id=\"LocalAppDataFolder\">\n        <Directory Id=\"UserPrograms\" Name=\"Programs\">\n          <Directory Id=\"INSTALLDIR\" Name=\"{folder}\" />\n        </Directory>\n      </Directory>\n"
        ));
    }
    w.push_str("      <Directory Id=\"ProgramMenuFolder\" />\n      <Directory Id=\"DesktopFolder\" />\n    </Directory>\n\n");

    w.push_str(&format!(
        "    <DirectoryRef Id=\"INSTALLDIR\">\n      <Component Id=\"BookFiles\" Guid=\"{}\" Win64=\"yes\">\n",
        component_guid(&c.meta.package_id, "BookFiles")
    ));
    let exe_src = x(&stage.exe.display().to_string());
    let pak_src = x(&stage.pak.display().to_string());
    let exe_name = x(&stage.exe_name);
    if per_machine {
        w.push_str(&format!("        <File Id=\"MainExe\" Name=\"{exe_name}\" Source=\"{exe_src}\" KeyPath=\"yes\" Checksum=\"yes\" />\n"));
        w.push_str(&format!("        <File Id=\"BookPak\" Name=\"book.pak\" Source=\"{pak_src}\" />\n"));
    } else {
        w.push_str(&format!("        <File Id=\"MainExe\" Name=\"{exe_name}\" Source=\"{exe_src}\" Checksum=\"yes\" />\n"));
        w.push_str(&format!("        <File Id=\"BookPak\" Name=\"book.pak\" Source=\"{pak_src}\" />\n"));
        w.push_str(&format!("        <RegistryValue Root=\"HKCU\" Key=\"{regkey}\" Name=\"InstallDir\" Type=\"string\" Value=\"[INSTALLDIR]\" KeyPath=\"yes\" />\n"));
        w.push_str("        <RemoveFolder Id=\"RemoveInstallDir\" Directory=\"INSTALLDIR\" On=\"uninstall\" />\n");
        w.push_str("        <RemoveFolder Id=\"RemoveUserPrograms\" Directory=\"UserPrograms\" On=\"uninstall\" />\n");
    }
    w.push_str("      </Component>\n");
    let mut refs = vec!["BookFiles"];
    let sm = component_guid(&c.meta.package_id, "StartMenuShortcut");
    let dk = component_guid(&c.meta.package_id, "DesktopShortcut");
    if b.start_menu_shortcut {
        w.push_str(&format!(
            "      <Component Id=\"StartMenuShortcut\" Guid=\"{sm}\" Win64=\"yes\">\n        <Shortcut Id=\"StartMenuLink\" Directory=\"ProgramMenuFolder\" Name=\"{folder}\" Target=\"[INSTALLDIR]{exe_name}\" WorkingDirectory=\"INSTALLDIR\" Icon=\"AppIcon.ico\" />\n        <RegistryValue Root=\"{root}\" Key=\"{regkey}\" Name=\"StartMenuShortcut\" Type=\"integer\" Value=\"1\" KeyPath=\"yes\" />\n      </Component>\n"
        ));
        refs.push("StartMenuShortcut");
    }
    if b.desktop_shortcut {
        w.push_str(&format!(
            "      <Component Id=\"DesktopShortcut\" Guid=\"{dk}\" Win64=\"yes\">\n        <Shortcut Id=\"DesktopLink\" Directory=\"DesktopFolder\" Name=\"{folder}\" Target=\"[INSTALLDIR]{exe_name}\" WorkingDirectory=\"INSTALLDIR\" Icon=\"AppIcon.ico\" />\n        <RegistryValue Root=\"{root}\" Key=\"{regkey}\" Name=\"DesktopShortcut\" Type=\"integer\" Value=\"1\" KeyPath=\"yes\" />\n      </Component>\n"
        ));
        refs.push("DesktopShortcut");
    }
    w.push_str("    </DirectoryRef>\n\n    <Feature Id=\"Main\" Title=\"Book\" Level=\"1\">\n");
    for r in &refs {
        w.push_str(&format!("      <ComponentRef Id=\"{r}\" />\n"));
    }
    w.push_str("    </Feature>\n");
    if let Some(rtf) = &stage.license_rtf {
        w.push_str(&format!("    <WixVariable Id=\"WixUILicenseRtf\" Value=\"{}\" />\n", x(&rtf.display().to_string())));
        w.push_str("    <UIRef Id=\"WixUI_Minimal\" />\n");
    }
    w.push_str("  </Product>\n</Wix>\n");

    let wxs = stage.work.join("installer.wxs");
    let obj = stage.work.join("installer.wixobj");
    std::fs::write(&wxs, w).context("Writing the WiX source")?;
    tools::run(
        Command::new(wix_bin.join("candle.exe")).arg("-nologo").arg("-arch").arg("x64").arg("-out").arg(&obj).arg(&wxs),
        "WiX candle",
    )?;
    let mut light = Command::new(wix_bin.join("light.exe"));
    light.arg("-nologo").arg("-cultures:en-us").arg("-sw1076").arg("-out").arg(out).arg(&obj);
    if stage.license_rtf.is_some() {
        light.arg("-ext").arg("WixUIExtension");
    }
    tools::run(&mut light, "WiX light")?;
    let _ = std::fs::remove_file(out.with_extension("wixpdb"));
    Ok(())
}

/// Minimal RTF for WiX's license page.
pub fn text_to_rtf(text: &str) -> String {
    let mut out = String::from("{\\rtf1\\ansi\\ansicpg1252\\deff0{\\fonttbl{\\f0 Segoe UI;}}\\fs18 ");
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\n' => out.push_str("\\par\n"),
            '\r' => {}
            c if (c as u32) < 128 => out.push(c),
            c => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{}?", *unit as i16));
                }
            }
        }
    }
    out.push('}');
    out
}
