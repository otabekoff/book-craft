//! `.msix` via MakeAppx. The package must be signed with a certificate whose
//! subject equals the manifest's Publisher before Windows will install it.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::Stage;
use crate::tools;

fn x(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace(['\r', '\n'], " ")
}

pub fn build(stage: &Stage, makeappx: &Path, out: &Path) -> Result<()> {
    let c = &stage.config;
    let layout = stage.work.join("msix");
    if layout.exists() {
        std::fs::remove_dir_all(&layout)?;
    }
    let assets = layout.join("Assets");
    std::fs::create_dir_all(&assets)?;
    std::fs::copy(&stage.exe, layout.join(&stage.exe_name))?;
    std::fs::copy(&stage.pak, layout.join("book.pak"))?;

    let icons = &stage.icons;
    std::fs::write(assets.join("StoreLogo.png"), icons.png(50)?)?;
    std::fs::write(assets.join("Square44x44Logo.png"), icons.png(44)?)?;
    std::fs::write(assets.join("Square44x44Logo.targetsize-256_altform-unplated.png"), icons.png(256)?)?;
    std::fs::write(assets.join("Square44x44Logo.targetsize-48_altform-unplated.png"), icons.png(48)?)?;
    std::fs::write(assets.join("Square44x44Logo.targetsize-32_altform-unplated.png"), icons.png(32)?)?;
    std::fs::write(assets.join("Square44x44Logo.targetsize-16_altform-unplated.png"), icons.png(16)?)?;
    std::fs::write(assets.join("Square71x71Logo.png"), icons.padded_png(71, 71, 0.8)?)?;
    std::fs::write(assets.join("Square150x150Logo.png"), icons.padded_png(150, 150, 0.66)?)?;
    std::fs::write(assets.join("Square310x310Logo.png"), icons.padded_png(310, 310, 0.6)?)?;
    std::fs::write(assets.join("Wide310x150Logo.png"), icons.padded_png(310, 150, 0.66)?)?;

    let name = &c.meta.package_id;
    let display = x(&c.product_name());
    let publisher_display = x(if c.meta.publisher.trim().is_empty() { &c.meta.author } else { &c.meta.publisher });
    let description = {
        let d = c.meta.description.trim();
        x(if d.is_empty() { &c.meta.title } else { d })
    };
    let lang = if c.meta.language.trim().is_empty() { "en-US".to_string() } else { x(c.meta.language.trim()) };
    let manifest = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
         IgnorableNamespaces="uap rescap">
  <Identity Name="{name}" Publisher="{publisher}" Version="{version}" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>{display}</DisplayName>
    <PublisherDisplayName>{publisher_display}</PublisherDisplayName>
    <Description>{description}</Description>
    <Logo>Assets\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Resources>
    <Resource Language="{lang}" />
  </Resources>
  <Applications>
    <Application Id="Book" Executable="{exe}" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="{display}" Description="{description}" BackgroundColor="transparent"
                          Square150x150Logo="Assets\Square150x150Logo.png" Square44x44Logo="Assets\Square44x44Logo.png">
        <uap:DefaultTile Wide310x150Logo="Assets\Wide310x150Logo.png" Square71x71Logo="Assets\Square71x71Logo.png" Square310x310Logo="Assets\Square310x310Logo.png" ShortName="{short}" />
      </uap:VisualElements>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
"#,
        publisher = x(c.meta.publisher_id.trim()),
        version = c.version4(),
        exe = x(&stage.exe_name),
        short = x(&c.product_name().chars().take(40).collect::<String>()),
    );
    std::fs::write(layout.join("AppxManifest.xml"), manifest).context("Writing AppxManifest.xml")?;
    tools::run(
        Command::new(makeappx).arg("pack").arg("/o").arg("/h").arg("SHA256").arg("/d").arg(&layout).arg("/p").arg(out),
        "MakeAppx",
    )?;
    Ok(())
}
