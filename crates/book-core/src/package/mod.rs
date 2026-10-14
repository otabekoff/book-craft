//! The whole build: book.pak → customised reader exe → the chosen packages.

pub mod msi;
pub mod msix;
pub mod nsis;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::config::BookConfig;
use crate::icons::IconSet;
use crate::project::Project;
use crate::tools::{self, ToolPaths};
use crate::{exe, pak};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Portable,
    Nsis,
    Msi,
    Msix,
}

impl Target {
    pub fn label(self) -> &'static str {
        match self {
            Target::Portable => "Portable .exe",
            Target::Nsis => "Setup .exe (NSIS)",
            Target::Msi => "Windows Installer .msi",
            Target::Msix => "MSIX package",
        }
    }
}

pub struct BuildRequest {
    pub targets: Vec<Target>,
    /// The prebuilt reader (`book-reader.exe`).
    pub reader_template: PathBuf,
    /// Folder with `katex/` and `mermaid.min.js`.
    pub vendor_dir: Option<PathBuf>,
    /// The bundled font library (`fonts.json` and one folder per font).
    pub fonts_dir: Option<PathBuf>,
    pub tools: ToolPaths,
    pub certificate_password: Option<String>,
    pub keep_work_files: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub target: Target,
    pub label: String,
    pub path: Option<String>,
    pub size: u64,
    pub signed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildReport {
    pub output_dir: String,
    pub artifacts: Vec<Artifact>,
    pub warnings: Vec<String>,
    pub seconds: f64,
    pub pak_size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub level: String,
    pub message: String,
}

pub type Log<'a> = dyn FnMut(LogLine) + 'a;

fn info(log: &mut Log, msg: impl Into<String>) {
    log(LogLine { level: "info".into(), message: msg.into() });
}
fn warn(log: &mut Log, msg: impl Into<String>) {
    log(LogLine { level: "warn".into(), message: msg.into() });
}

/// Everything the installer builders need, already written to the work folder.
pub struct Stage {
    pub config: BookConfig,
    pub work: PathBuf,
    pub exe: PathBuf,
    pub exe_name: String,
    pub pak: PathBuf,
    pub ico: PathBuf,
    pub icons: IconSet,
    pub license_txt: Option<PathBuf>,
    pub license_rtf: Option<PathBuf>,
    pub webview2: Option<PathBuf>,
    /// Product name safe for folder and shortcut names.
    pub product_file_name: String,
}

fn file_safe(name: &str) -> String {
    let s: String = name.chars().map(|c| if "\\/:*?\"<>|".contains(c) || c.is_control() { ' ' } else { c }).collect();
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = s.trim_matches('.').trim().to_string();
    if s.is_empty() { "Book".into() } else { s }
}

struct Signer<'a> {
    signtool: Option<PathBuf>,
    config: &'a BookConfig,
    password: Option<&'a str>,
}

impl Signer<'_> {
    fn enabled(&self) -> bool {
        self.config.build.signing.enabled
    }

    fn sign(&self, file: &Path, log: &mut Log) -> Result<bool> {
        if !self.enabled() {
            return Ok(false);
        }
        let tool = self.signtool.as_ref().ok_or_else(|| anyhow!("SignTool was not found (install the Windows SDK)."))?;
        let s = &self.config.build.signing;
        let cert = s.certificate.trim();
        if cert.is_empty() || !Path::new(cert).is_file() {
            bail!("The signing certificate {cert:?} was not found.");
        }
        let mut cmd = Command::new(tool);
        cmd.arg("sign").arg("/fd").arg("SHA256").arg("/f").arg(cert);
        if let Some(p) = self.password.filter(|p| !p.is_empty()) {
            cmd.arg("/p").arg(p);
        }
        if !s.timestamp_url.trim().is_empty() {
            cmd.arg("/tr").arg(s.timestamp_url.trim()).arg("/td").arg("SHA256");
        }
        cmd.arg("/d").arg(self.config.product_name()).arg(file);
        tools::run(&mut cmd, "SignTool")?;
        info(log, format!("Signed {}", file.file_name().unwrap_or_default().to_string_lossy()));
        Ok(true)
    }
}

pub fn build(project: &Project, req: &BuildRequest, log: &mut Log) -> Result<BuildReport> {
    let started = Instant::now();
    let config = &project.config;
    let problems = config.validate();
    if !problems.is_empty() {
        bail!("Please fix these first:\n• {}", problems.join("\n• "));
    }
    if req.targets.is_empty() {
        bail!("Choose at least one package to build.");
    }
    if !req.reader_template.is_file() {
        bail!(
            "The reader template was not found at {}. Run `bun run prepare` (or build the book-reader crate) first.",
            req.reader_template.display()
        );
    }

    let out_dir = project.output_dir()?;
    // Packaging tools still trip over paths longer than 260 characters, so all
    // intermediate files live in a short temp folder and results are moved at the end.
    let work = work_dir(&out_dir);
    if work.exists() {
        std::fs::remove_dir_all(&work).with_context(|| format!("Cleaning {}", work.display()))?;
    }
    std::fs::create_dir_all(work.join("stage"))?;
    std::fs::create_dir_all(work.join("out"))?;
    let mut warnings = Vec::new();

    // 1. Icons.
    let icons = IconSet::load(project)?;
    let ico = work.join("app.ico");
    std::fs::write(&ico, icons.ico()?)?;
    info(log, "Prepared icons");

    // 2. The book itself.
    let pak_path = work.join("stage").join(pak_format::PAK_FILE);
    let mut pak_log = |m: &str| info(log, m.to_string());
    let library = req.fonts_dir.as_deref().and_then(crate::fonts::FontLibrary::load);
    let pak = pak::build(project, req.vendor_dir.as_deref(), library.as_ref(), &icons.png(256)?, &pak_path, &mut pak_log)?;
    for w in &pak.warnings {
        warn(log, w.clone());
    }
    warnings.extend(pak.warnings.iter().cloned());
    info(
        log,
        format!(
            "Packed {} chapter(s), {} words, {:.1} MB",
            pak.manifest.chapters.len(),
            pak.manifest.words,
            pak.size as f64 / 1_048_576.0
        ),
    );

    // 3. The executable.
    let exe_name = format!("{}.exe", config.exe_stem());
    let exe_path = work.join("stage").join(&exe_name);
    exe::customize(&req.reader_template, &exe_path, config, &icons)?;
    info(log, format!("Created {exe_name} with the book's icon and details"));
    if config.build.signing.enabled {
        let cert = std::path::Path::new(config.build.signing.certificate.trim());
        let cert_info = crate::signing::inspect(cert, req.certificate_password.as_deref())
            .context("Checking the signing certificate")?;
        let problems = crate::signing::check_for_build(&cert_info, &config.meta.publisher_id, req.targets.contains(&Target::Msix));
        if !problems.is_empty() {
            bail!("The signing certificate can't be used:\n• {}", problems.join("\n• "));
        }
        info(log, format!("Signing as {} (valid until {})", cert_info.subject, cert_info.not_after));
        if cert_info.self_signed {
            warn(log, "This is a self-signed test certificate: fine for testing, but readers' PCs won't trust it.");
        }
    }
    let signer = Signer {

        signtool: req.tools.signtool(),
        config,
        password: req.certificate_password.as_deref(),
    };
    let mut exe_signed = false;
    if signer.enabled() {
        exe_signed = signer.sign(&exe_path, log)?;
    } else {
        warn(log, "Not signed: Windows SmartScreen may warn readers before the book opens.");
    }

    // License for the installers.
    let (license_txt, license_rtf) = if config.build.license_file.trim().is_empty() {
        (None, None)
    } else {
        let src = project.resolve(&config.build.license_file)?;
        let text = std::fs::read_to_string(&src).with_context(|| format!("Reading the license {}", src.display()))?;
        let txt = work.join("license.txt");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend(text.replace("\r\n", "\n").replace('\n', "\r\n").as_bytes());
        std::fs::write(&txt, bytes)?;
        let rtf = work.join("license.rtf");
        std::fs::write(&rtf, msi::text_to_rtf(&text))?;
        (Some(txt), Some(rtf))
    };

    let webview2 = if config.build.webview2 == "embed" {
        let p = req.tools.webview2_bootstrapper();
        if p.is_none() {
            warn(log, "WebView2 bootstrapper not found; the setup will download it if needed.");
        }
        p
    } else {
        None
    };

    let stage = Stage {
        config: config.clone(),
        work: work.clone(),
        exe: exe_path.clone(),
        exe_name: exe_name.clone(),
        pak: pak_path.clone(),
        ico,
        icons,
        license_txt,
        license_rtf,
        webview2,
        product_file_name: file_safe(&config.product_name()),
    };

    std::fs::create_dir_all(&out_dir)?;
    let base = format!("{}-{}", config.exe_stem().replace(' ', "-"), config.version4());
    let mut artifacts = Vec::new();
    for &target in &req.targets {
        info(log, format!("Building {}…", target.label()));
        let name = match target {
            Target::Portable => format!("{base}-portable.exe"),
            Target::Nsis => format!("{base}-setup.exe"),
            Target::Msi => format!("{base}.msi"),
            Target::Msix => format!("{base}.msix"),
        };
        let file = work.join("out").join(&name);
        let dest = out_dir.join(&name);
        let result: Result<bool> = (|| {
            if dest.exists() {
                std::fs::remove_file(&dest).with_context(|| format!("{} is in use — close it and try again", dest.display()))?;
            }
            match target {
                Target::Portable => {
                    std::fs::copy(&exe_path, &file)?;
                    if exe_signed {
                        // Re-sign after appending so the signature covers the book.
                        strip_signature_copy(&req.reader_template, &stage, &file)?;
                    }
                    pak_format::append_file(&file, &pak_path)?;
                    signer.sign(&file, log)
                }
                Target::Nsis => {
                    let tool = req.tools.makensis().ok_or_else(|| anyhow!("NSIS was not found (see Tools)."))?;
                    nsis::build(&stage, &tool, &file)?;
                    signer.sign(&file, log)
                }
                Target::Msi => {
                    let wix = req.tools.wix().ok_or_else(|| anyhow!("WiX Toolset 3 was not found (see Tools)."))?;
                    msi::build(&stage, &wix, &file)?;
                    signer.sign(&file, log)
                }
                Target::Msix => {
                    let tool = req.tools.makeappx().ok_or_else(|| anyhow!("MakeAppx was not found (install the Windows SDK)."))?;
                    msix::build(&stage, &tool, &file)?;
                    let signed = signer.sign(&file, log)?;
                    if !signed {
                        warn(log, "The MSIX is unsigned. Windows installs only signed MSIX packages (the Store signs Store submissions for you).");
                    }
                    Ok(signed)
                }
            }
        })();
        let result = result.and_then(|signed| {
            move_file(&file, &dest)?;
            Ok(signed)
        });
        match result {
            Ok(signed) => {
                let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                info(log, format!("✓ {} — {:.1} MB", dest.display(), size as f64 / 1_048_576.0));
                artifacts.push(Artifact {
                    target,
                    label: target.label().into(),
                    path: Some(dest.display().to_string()),
                    size,
                    signed,
                    error: None,
                });
            }
            Err(e) => {
                let msg = format!("{e:#}");
                log(LogLine { level: "error".into(), message: format!("✗ {}: {msg}", target.label()) });
                artifacts.push(Artifact { target, label: target.label().into(), path: None, size: 0, signed: false, error: Some(msg) });
            }
        }
    }

    if !req.keep_work_files {
        let _ = std::fs::remove_dir_all(&work);
    }
    Ok(BuildReport {
        output_dir: out_dir.display().to_string(),
        artifacts,
        warnings,
        seconds: started.elapsed().as_secs_f64(),
        pak_size: pak.size,
    })
}

fn work_dir(out_dir: &Path) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    out_dir.hash(&mut h);
    std::env::temp_dir().join("bookcraft").join(format!("{:016x}", h.finish()))
}

fn move_file(from: &Path, to: &Path) -> Result<()> {
    if std::fs::rename(from, to).is_err() {
        std::fs::copy(from, to).with_context(|| format!("Copying to {}", to.display()))?;
        let _ = std::fs::remove_file(from);
    }
    Ok(())
}

/// For a signed portable build, start again from the unsigned customised exe
/// so the final signature covers the appended book.
fn strip_signature_copy(template: &Path, stage: &Stage, file: &Path) -> Result<()> {
    exe::customize(template, file, &stage.config, &stage.icons)
}
