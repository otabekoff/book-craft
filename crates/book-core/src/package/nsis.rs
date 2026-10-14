//! Setup `.exe` via NSIS (Modern UI 2).

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::Stage;
use crate::tools;

/// NSIS string literal contents: `$` and quotes are escaped, newlines removed.
fn ns(s: &str) -> String {
    s.replace('$', "$$").replace('"', "$\\\"").replace(['\r', '\n'], " ")
}

pub fn build(stage: &Stage, makensis: &Path, out: &Path) -> Result<()> {
    let c = &stage.config;
    let b = &c.build;
    let per_machine = b.install_mode == "perMachine";
    let product = ns(&stage.product_file_name);
    let script_path = stage.work.join("installer.nsi");

    let mut s = String::new();
    s.push_str("Unicode true\nManifestDPIAware true\nSetCompressor /SOLID lzma\n");
    s.push_str("!include \"MUI2.nsh\"\n!include \"FileFunc.nsh\"\n!include \"LogicLib.nsh\"\n\n");
    s.push_str(&format!("!define PRODUCT \"{}\"\n", ns(&c.product_name())));
    s.push_str(&format!("!define PRODUCTFILE \"{product}\"\n"));
    s.push_str(&format!("!define PUBLISHER \"{}\"\n", ns(&c.meta.publisher)));
    s.push_str(&format!("!define VERSION \"{}\"\n", c.version4()));
    s.push_str(&format!("!define EXE \"{}\"\n", ns(&stage.exe_name)));
    s.push_str(&format!(
        "!define UNINSTKEY \"Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}\"\n",
        ns(&c.meta.package_id)
    ));
    s.push_str(&format!("!define APPKEY \"Software\\{}\"\n\n", ns(&c.meta.package_id)));

    s.push_str("Name \"${PRODUCT}\"\n");
    s.push_str(&format!("OutFile \"{}\"\n", ns(&out.display().to_string())));
    if per_machine {
        s.push_str("InstallDir \"$PROGRAMFILES64\\${PRODUCTFILE}\"\nInstallDirRegKey HKLM \"${APPKEY}\" \"InstallDir\"\nRequestExecutionLevel admin\n");
    } else {
        s.push_str("InstallDir \"$LOCALAPPDATA\\Programs\\${PRODUCTFILE}\"\nInstallDirRegKey HKCU \"${APPKEY}\" \"InstallDir\"\nRequestExecutionLevel user\n");
    }
    s.push_str("BrandingText \"${PUBLISHER}\"\nShowInstDetails nevershow\nShowUninstDetails nevershow\n\n");

    s.push_str(&format!("!define MUI_ICON \"{}\"\n!define MUI_UNICON \"{}\"\n", ns(&stage.ico.display().to_string()), ns(&stage.ico.display().to_string())));
    s.push_str("!define MUI_ABORTWARNING\n");
    if b.launch_after_install {
        s.push_str("!define MUI_FINISHPAGE_RUN \"$INSTDIR\\${EXE}\"\n");
    }
    s.push_str("!insertmacro MUI_PAGE_WELCOME\n");
    if let Some(lic) = &stage.license_txt {
        s.push_str(&format!("!insertmacro MUI_PAGE_LICENSE \"{}\"\n", ns(&lic.display().to_string())));
    }
    s.push_str("!insertmacro MUI_PAGE_DIRECTORY\n!insertmacro MUI_PAGE_INSTFILES\n!insertmacro MUI_PAGE_FINISH\n");
    s.push_str("!insertmacro MUI_UNPAGE_CONFIRM\n!insertmacro MUI_UNPAGE_COMPONENTS\n!insertmacro MUI_UNPAGE_INSTFILES\n");
    s.push_str("!insertmacro MUI_LANGUAGE \"English\"\n\n");

    let v = c.version4();
    s.push_str(&format!("VIProductVersion \"{v}\"\nVIFileVersion \"{v}\"\n"));
    for (k, val) in [
        ("ProductName", c.product_name()),
        ("CompanyName", c.meta.publisher.clone()),
        ("LegalCopyright", c.meta.copyright.clone()),
        ("FileDescription", format!("{} Setup", c.product_name())),
        ("FileVersion", v.clone()),
        ("ProductVersion", v.clone()),
    ] {
        if !val.trim().is_empty() {
            s.push_str(&format!("VIAddVersionKey /LANG=${{LANG_ENGLISH}} \"{k}\" \"{}\"\n", ns(&val)));
        }
    }
    s.push('\n');

    // WebView2: Windows 11 has it; older Windows 10 machines may not.
    s.push_str("Function EnsureWebView2\n");
    s.push_str("  ReadRegStr $0 HKLM \"SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}\" \"pv\"\n");
    s.push_str("  ${If} $0 == \"\"\n    ReadRegStr $0 HKCU \"Software\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}\" \"pv\"\n  ${EndIf}\n");
    s.push_str("  ${If} $0 == \"\"\n  ${OrIf} $0 == \"0.0.0.0\"\n    InitPluginsDir\n");
    match (b.webview2.as_str(), &stage.webview2) {
        ("skip", _) => s.push_str("    DetailPrint \"WebView2 is missing; the book needs it to open.\"\n"),
        ("embed", Some(boot)) => {
            s.push_str("    DetailPrint \"Installing Microsoft Edge WebView2…\"\n");
            s.push_str("    SetOutPath \"$PLUGINSDIR\"\n");
            s.push_str(&format!("    File \"/oname=wv2setup.exe\" \"{}\"\n", ns(&boot.display().to_string())));
            s.push_str("    ExecWait '\"$PLUGINSDIR\\wv2setup.exe\" /silent /install' $1\n");
        }
        _ => {
            s.push_str("    DetailPrint \"Downloading Microsoft Edge WebView2…\"\n");
            s.push_str("    NSISdl::download \"https://go.microsoft.com/fwlink/p/?LinkId=2124703\" \"$PLUGINSDIR\\wv2setup.exe\"\n    Pop $1\n");
            s.push_str("    ${If} $1 == \"success\"\n      ExecWait '\"$PLUGINSDIR\\wv2setup.exe\" /silent /install' $1\n    ${EndIf}\n");
        }
    }
    s.push_str("  ${EndIf}\nFunctionEnd\n\n");

    s.push_str("Section \"Install\"\n");
    s.push_str(if per_machine { "  SetShellVarContext all\n" } else { "  SetShellVarContext current\n" });
    s.push_str("  SetOutPath \"$INSTDIR\"\n");
    s.push_str(&format!("  File \"/oname=${{EXE}}\" \"{}\"\n", ns(&stage.exe.display().to_string())));
    s.push_str(&format!("  File \"/oname=book.pak\" \"{}\"\n", ns(&stage.pak.display().to_string())));
    s.push_str("  WriteUninstaller \"$INSTDIR\\uninstall.exe\"\n");
    s.push_str("  Call EnsureWebView2\n");
    s.push_str("  SetOutPath \"$INSTDIR\"\n");
    if b.start_menu_shortcut {
        s.push_str("  CreateShortcut \"$SMPROGRAMS\\${PRODUCTFILE}.lnk\" \"$INSTDIR\\${EXE}\"\n");
    }
    if b.desktop_shortcut {
        s.push_str("  CreateShortcut \"$DESKTOP\\${PRODUCTFILE}.lnk\" \"$INSTDIR\\${EXE}\"\n");
    }
    s.push_str("  WriteRegStr SHCTX \"${APPKEY}\" \"InstallDir\" \"$INSTDIR\"\n");
    for (k, v) in [
        ("DisplayName", "${PRODUCT}"),
        ("DisplayVersion", "${VERSION}"),
        ("Publisher", "${PUBLISHER}"),
        ("DisplayIcon", "$INSTDIR\\${EXE}"),
        ("InstallLocation", "$INSTDIR"),
        ("UninstallString", "$\\\"$INSTDIR\\uninstall.exe$\\\""),
        ("QuietUninstallString", "$\\\"$INSTDIR\\uninstall.exe$\\\" /S"),
    ] {
        s.push_str(&format!("  WriteRegStr SHCTX \"${{UNINSTKEY}}\" \"{k}\" \"{v}\"\n"));
    }
    if !c.meta.website.trim().is_empty() {
        s.push_str(&format!("  WriteRegStr SHCTX \"${{UNINSTKEY}}\" \"URLInfoAbout\" \"{}\"\n", ns(&c.meta.website)));
    }
    s.push_str("  WriteRegDWORD SHCTX \"${UNINSTKEY}\" \"NoModify\" 1\n  WriteRegDWORD SHCTX \"${UNINSTKEY}\" \"NoRepair\" 1\n");
    s.push_str("  ${GetSize} \"$INSTDIR\" \"/S=0K\" $0 $1 $2\n  IntFmt $0 \"0x%08X\" $0\n  WriteRegDWORD SHCTX \"${UNINSTKEY}\" \"EstimatedSize\" \"$0\"\n");
    s.push_str("SectionEnd\n\n");

    s.push_str("Section \"Uninstall\"\n");
    s.push_str(if per_machine { "  SetShellVarContext all\n" } else { "  SetShellVarContext current\n" });
    s.push_str("  Delete \"$INSTDIR\\${EXE}\"\n  Delete \"$INSTDIR\\book.pak\"\n  Delete \"$INSTDIR\\uninstall.exe\"\n  RMDir \"$INSTDIR\"\n");
    s.push_str("  Delete \"$SMPROGRAMS\\${PRODUCTFILE}.lnk\"\n  Delete \"$DESKTOP\\${PRODUCTFILE}.lnk\"\n");
    s.push_str("  DeleteRegKey SHCTX \"${UNINSTKEY}\"\n  DeleteRegKey SHCTX \"${APPKEY}\"\n");
    s.push_str("SectionEnd\n\n");

    // Reading data lives in the reader's per-user folders (WebView2 profile, position,
    // bookmarks). Unticked by default, so silent uninstalls (/S) and upgrades keep it.
    let data = ns(&c.meta.package_id);
    s.push_str("Section /o \"un.Delete my reading position and bookmarks\" UnReadingData\n");
    s.push_str("  SetShellVarContext current\n");
    s.push_str(&format!("  RMDir /r \"$LOCALAPPDATA\\{data}\"\n  RMDir /r \"$APPDATA\\{data}\"\n"));
    s.push_str("SectionEnd\n");

    // UTF-8 with BOM so NSIS reads non-ASCII titles correctly.
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(s.as_bytes());
    std::fs::write(&script_path, bytes).context("Writing the NSIS script")?;
    tools::run(Command::new(makensis).arg("/V2").arg("/INPUTCHARSET").arg("UTF8").arg(&script_path), "NSIS")?;
    Ok(())
}
