//! Turns the prebuilt reader template into this book's executable:
//! new icon and version resource (name, publisher, version, copyright).

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use editpe::types::{VersionU16, VersionU32};
use editpe::{Image, VersionInfo, VersionStringTable};

use crate::config::BookConfig;
use crate::icons::IconSet;

pub fn customize(template: &Path, out: &Path, config: &BookConfig, icons: &IconSet) -> Result<()> {
    let data = std::fs::read(template).with_context(|| format!("Reading the reader template {}", template.display()))?;
    let mut image = Image::parse(data.as_slice()).map_err(|e| anyhow!("The reader template is not a valid .exe: {e}"))?;
    let mut res = image.resource_directory().cloned().unwrap_or_default();

    res.remove_main_icon().map_err(|e| anyhow!("Removing the template icon: {e}"))?;
    res.set_main_icon(icons.image.clone()).map_err(|e| anyhow!("Setting the icon: {e}"))?;

    let mut info = res.get_version_info().map_err(|e| anyhow!("Reading version info: {e}"))?.unwrap_or_default();
    let v = config.version_parts();
    let packed = VersionU32 {
        major: ((v[0] as u32) << 16) | v[1] as u32,
        minor: ((v[2] as u32) << 16) | v[3] as u32,
    };
    info.info.file_version = packed;
    info.info.product_version = packed;
    let version = config.version4();
    let exe_name = format!("{}.exe", config.exe_stem());
    let description = if config.meta.subtitle.trim().is_empty() {
        config.product_name()
    } else {
        format!("{} — {}", config.product_name(), config.meta.subtitle.trim())
    };
    let values: Vec<(&str, String)> = vec![
        ("CompanyName", config.meta.publisher.clone()),
        ("FileDescription", description),
        ("FileVersion", version.clone()),
        ("InternalName", config.exe_stem()),
        ("LegalCopyright", config.meta.copyright.clone()),
        ("OriginalFilename", exe_name),
        ("ProductName", config.product_name()),
        ("ProductVersion", version),
        ("Comments", format!("By {}. Made with Book Craft.", config.meta.author)),
    ];
    if info.strings.is_empty() {
        info.strings.push(VersionStringTable { key: "040904b0".into(), ..Default::default() });
    }
    if info.vars.is_empty() {
        info.vars.push(VersionU16 { major: 0x0409, minor: 1200 });
    }
    for table in &mut info.strings {
        for (k, val) in &values {
            table.strings.insert((*k).to_string(), val.clone());
        }
    }
    res.set_version_info(&info).map_err(|e| anyhow!("Writing version info: {e}"))?;
    image.set_resource_directory(res).map_err(|e| anyhow!("Updating resources: {e}"))?;
    image.write_file(out).map_err(|e| anyhow!("Writing {}: {e}", out.display()))?;
    Ok(())
}

/// Reads the version strings back (used by tests and the build log).
pub fn read_strings(path: &Path) -> Result<Vec<(String, String)>> {
    let data = std::fs::read(path)?;
    let image = Image::parse(data.as_slice()).map_err(|e| anyhow!("{e}"))?;
    let res = image.resource_directory().ok_or_else(|| anyhow!("no resources"))?;
    let info: VersionInfo = res.get_version_info().map_err(|e| anyhow!("{e}"))?.ok_or_else(|| anyhow!("no version info"))?;
    Ok(info.strings.iter().flat_map(|t| t.strings.iter().map(|(k, v)| (k.clone(), v.clone()))).collect())
}
