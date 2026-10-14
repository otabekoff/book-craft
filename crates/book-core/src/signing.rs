//! Code-signing certificates: inspect a .pfx, create a self-signed test
//! certificate, and (on request, with a UAC prompt) trust one for MSIX installs.
//!
//! Passwords travel to PowerShell through an environment variable, never on
//! the command line, so they don't show up in process listings.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub thumbprint: String,
    /// ISO date.
    pub not_after: String,
    pub expired: bool,
    pub self_signed: bool,
    pub code_signing: bool,
    pub has_private_key: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCertificate {
    pub pfx: String,
    pub cer: String,
    pub info: CertificateInfo,
}

fn powershell(script: &str, password: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("powershell.exe");
    // A PowerShell 7 parent passes its module path down, which hides Windows PowerShell's Cert: drive.
    cmd.env_remove("PSModulePath");
    let script = format!("Import-Module Microsoft.PowerShell.Security, PKI -ErrorAction SilentlyContinue
{script}");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script]);
    if let Some(p) = password {
        cmd.env("BOOKCRAFT_PFX_PASSWORD", p);
    }
    crate::tools::run(&mut cmd, "PowerShell").map(|s| s.trim().to_string())
}

fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Reads a .pfx (or .cer) and describes it.
pub fn inspect(path: &Path, password: Option<&str>) -> Result<CertificateInfo> {
    if !path.is_file() {
        bail!("{} was not found", path.display());
    }
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
$pw = $env:BOOKCRAFT_PFX_PASSWORD
$flags = [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::EphemeralKeySet
try {{ $c = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new({path}, $pw, $flags) }}
catch {{ throw "The certificate could not be opened. Check the password." }}
$eku = @($c.Extensions | Where-Object {{ $_ -is [System.Security.Cryptography.X509Certificates.X509EnhancedKeyUsageExtension] }} | ForEach-Object {{ $_.EnhancedKeyUsages }} | ForEach-Object {{ $_.Value }})
[pscustomobject]@{{
  subject = $c.Subject; issuer = $c.Issuer; thumbprint = $c.Thumbprint
  notAfter = $c.NotAfter.ToString('yyyy-MM-dd'); expired = ($c.NotAfter -lt (Get-Date))
  selfSigned = ($c.Subject -eq $c.Issuer)
  codeSigning = ($eku.Count -eq 0 -or $eku -contains '1.3.6.1.5.5.7.3.3')
  hasPrivateKey = $c.HasPrivateKey
}} | ConvertTo-Json -Compress"#,
        path = ps_quote(&path.display().to_string())
    );
    let out = powershell(&script, password)?;
    serde_json::from_str(&out).with_context(|| format!("Unexpected certificate details: {out}"))
}

/// Creates a self-signed code-signing certificate for testing. It is made in the
/// user's certificate store, exported to `dir`, then removed from the store again.
pub fn create_test_certificate(subject: &str, password: &str, dir: &Path) -> Result<TestCertificate> {
    let subject = subject.trim();
    if !subject.starts_with("CN=") {
        bail!("The subject must start with CN=, for example CN=Harbourlight Press");
    }
    if password.len() < 4 {
        bail!("Use a password of at least 4 characters to protect the certificate file.");
    }
    std::fs::create_dir_all(dir)?;
    let stem = subject[3..]
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let stem = if stem.is_empty() { "test".to_string() } else { stem };
    let pfx: PathBuf = dir.join(format!("{stem}-test-signing.pfx"));
    let cer: PathBuf = dir.join(format!("{stem}-test-signing.cer"));
    let script = format!(
        r#"$ErrorActionPreference = 'Stop'
$c = New-SelfSignedCertificate -Type Custom -Subject {subject} -KeyUsage DigitalSignature -FriendlyName 'Book Craft test signing' `
  -CertStoreLocation 'Cert:\CurrentUser\My' -NotAfter (Get-Date).AddYears(2) `
  -TextExtension @('2.5.29.37={{text}}1.3.6.1.5.5.7.3.3', '2.5.29.19={{text}}')
try {{
  $pw = ConvertTo-SecureString -String $env:BOOKCRAFT_PFX_PASSWORD -Force -AsPlainText
  Export-PfxCertificate -Cert $c -FilePath {pfx} -Password $pw | Out-Null
  Export-Certificate -Cert $c -FilePath {cer} | Out-Null
}} finally {{
  Remove-Item -LiteralPath ('Cert:\CurrentUser\My\' + $c.Thumbprint) -DeleteKey -ErrorAction SilentlyContinue
}}
'ok'"#,
        subject = ps_quote(subject),
        pfx = ps_quote(&pfx.display().to_string()),
        cer = ps_quote(&cer.display().to_string()),
    );
    powershell(&script, Some(password))?;
    let info = inspect(&pfx, Some(password))?;
    Ok(TestCertificate { pfx: pfx.display().to_string(), cer: cer.display().to_string(), info })
}

/// Asks Windows (UAC prompt) to trust a certificate for installing MSIX packages
/// on this PC, by adding it to Local Machine › Trusted People. Only for test certificates.
pub fn trust_for_msix(cer: &Path) -> Result<()> {
    if !cer.is_file() {
        bail!("{} was not found", cer.display());
    }
    let inner = format!(
        "Import-Certificate -FilePath {} -CertStoreLocation Cert:\\LocalMachine\\TrustedPeople | Out-Null",
        ps_quote(&cer.display().to_string())
    );
    let script = format!(
        "$p = Start-Process powershell.exe -Verb RunAs -Wait -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile','-Command',{}; exit $p.ExitCode",
        ps_quote(&inner)
    );
    powershell(&script, None).context("Windows did not trust the certificate (was the prompt cancelled?)")?;
    Ok(())
}

/// Problems that would make signing fail or the MSIX refuse to install.
pub fn check_for_build(info: &CertificateInfo, publisher_id: &str, msix: bool) -> Vec<String> {
    let mut out = Vec::new();
    if info.expired {
        out.push(format!("The certificate expired on {}.", info.not_after));
    }
    if !info.has_private_key {
        out.push("The certificate file has no private key, so it cannot sign. Export it as .pfx with the key.".into());
    }
    if !info.code_signing {
        out.push("The certificate is not meant for code signing.".into());
    }
    if msix && normalize_dn(&info.subject) != normalize_dn(publisher_id) {
        out.push(format!(
            "MSIX needs the Publisher ID to equal the certificate subject. Certificate: {} — Publisher ID: {}.",
            info.subject, publisher_id
        ));
    }
    out
}

fn normalize_dn(s: &str) -> String {
    let mut parts: Vec<String> = s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect();
    parts.sort();
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(subject: &str) -> CertificateInfo {
        CertificateInfo {
            subject: subject.into(),
            issuer: subject.into(),
            thumbprint: "X".into(),
            not_after: "2030-01-01".into(),
            expired: false,
            self_signed: true,
            code_signing: true,
            has_private_key: true,
        }
    }

    #[test]
    fn publisher_must_match_for_msix() {
        assert!(check_for_build(&info("CN=A, O=B"), "O=B,CN=A", true).is_empty());
        assert_eq!(check_for_build(&info("CN=A"), "CN=B", true).len(), 1);
        assert!(check_for_build(&info("CN=A"), "CN=B", false).is_empty());
    }

    #[test]
    #[ignore = "creates and removes a certificate in the user's store; run with --ignored"]
    fn creates_test_certificate() {
        let dir = tempfile::tempdir().unwrap();
        let cert = create_test_certificate("CN=Book Craft Test", "test-pass", dir.path()).unwrap();
        assert_eq!(cert.info.subject, "CN=Book Craft Test");
        assert!(cert.info.code_signing && cert.info.has_private_key);
        assert!(inspect(Path::new(&cert.pfx), Some("wrong")).is_err());
    }
}
