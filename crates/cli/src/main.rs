//! `bookcraft` — build book apps from the command line (CI, scripts, testing).
//!
//! ```text
//! bookcraft new <folder> [--title "My Book"] [--author "Name"]
//! bookcraft build <folder> [--targets portable,nsis,msi,msix] [--reader <exe>] [--vendor <dir>] [--password <pfx password>] [--keep]
//! bookcraft render <file.md>          print chapter HTML
//! bookcraft tools                     show which packaging tools were found
//! bookcraft cert new "CN=Name" <folder> --password <pw>   make a self-signed test certificate
//! bookcraft cert show <file.pfx> [--password <pw>]
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use book_core::Project;
use book_core::package::{BuildRequest, Target, build};
use book_core::render::{RenderOptions, render_markdown};
use book_core::tools::ToolPaths;

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("new") => cmd_new(&args),
        Some("build") => cmd_build(&args),
        Some("render") => cmd_render(&args),
        Some("cert") => cmd_cert(&args),
        Some("tools") => {
            for t in ToolPaths::default().status() {
                println!("{:<24} {:<34} {}", t.name, t.needed_for, t.path.unwrap_or_else(|| format!("missing — {}", t.help)));
            }
            Ok(())
        }
        _ => {
            eprintln!("usage: bookcraft new|build|render|tools … (see crates/cli/src/main.rs)");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_new(args: &[String]) -> anyhow::Result<()> {
    let dir = args.get(1).ok_or_else(|| anyhow::anyhow!("missing folder"))?;
    let title = flag(args, "--title").unwrap_or_else(|| "My Book".into());
    let author = flag(args, "--author").unwrap_or_default();
    std::fs::create_dir_all(dir)?;
    let p = Project::create(dir, &title, &author)?;
    println!("Created {} with {} chapters", p.root.display(), p.config.chapters.len());
    Ok(())
}

fn cmd_render(args: &[String]) -> anyhow::Result<()> {
    let file = args.get(1).ok_or_else(|| anyhow::anyhow!("missing file"))?;
    let md = std::fs::read_to_string(file)?;
    let config = book_core::BookConfig::default();
    let r = render_markdown(&md, &RenderOptions { config: &config, asset_base: "/".into(), chapter_file: None });
    println!("{}", r.html);
    for w in r.warnings {
        eprintln!("warning: {w}");
    }
    Ok(())
}

fn cmd_build(args: &[String]) -> anyhow::Result<()> {
    let dir = args.get(1).ok_or_else(|| anyhow::anyhow!("missing project folder"))?;
    let project = Project::open(dir)?;
    let b = &project.config.build;
    let targets = match flag(args, "--targets") {
        Some(list) => list
            .split(',')
            .map(|t| match t.trim() {
                "portable" | "exe" => Ok(Target::Portable),
                "nsis" | "setup" => Ok(Target::Nsis),
                "msi" => Ok(Target::Msi),
                "msix" => Ok(Target::Msix),
                other => Err(anyhow::anyhow!("unknown target {other}")),
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        None => [(b.portable, Target::Portable), (b.nsis, Target::Nsis), (b.msi, Target::Msi), (b.msix, Target::Msix)]
            .into_iter()
            .filter_map(|(on, t)| on.then_some(t))
            .collect(),
    };
    let root = workspace_root();
    let reader = flag(args, "--reader").map(PathBuf::from).unwrap_or_else(|| {
        let bundled = root.join("src-tauri/resources/reader/book-reader.exe");
        if bundled.is_file() { bundled } else { root.join("target/release/book-reader.exe") }
    });
    let vendor = flag(args, "--vendor").map(PathBuf::from).unwrap_or_else(|| root.join("src-tauri/resources/runtime/vendor"));
    let req = BuildRequest {
        targets,
        reader_template: reader,
        vendor_dir: Some(vendor),
        fonts_dir: Some(root.join("src-tauri/resources/runtime/fonts")),
        tools: ToolPaths::default(),
        certificate_password: flag(args, "--password").or_else(|| std::env::var("BOOKCRAFT_CERT_PASSWORD").ok()),
        keep_work_files: args.iter().any(|a| a == "--keep"),
    };
    let report = build(&project, &req, &mut |line| {
        println!("[{}] {}", line.level, line.message);
    })?;
    println!("\nDone in {:.1}s → {}", report.seconds, report.output_dir);
    let mut failed = false;
    for a in &report.artifacts {
        match (&a.path, &a.error) {
            (Some(p), _) => println!("  ✓ {:<26} {} ({:.1} MB){}", a.label, p, a.size as f64 / 1_048_576.0, if a.signed { ", signed" } else { "" }),
            (None, Some(e)) => {
                failed = true;
                println!("  ✗ {:<26} {e}", a.label)
            }
            _ => {}
        }
    }
    if failed {
        anyhow::bail!("some packages failed");
    }
    Ok(())
}

fn cmd_cert(args: &[String]) -> anyhow::Result<()> {
    let password = flag(args, "--password").or_else(|| std::env::var("BOOKCRAFT_CERT_PASSWORD").ok());
    match args.get(1).map(String::as_str) {
        Some("new") => {
            let subject = args.get(2).ok_or_else(|| anyhow::anyhow!("missing subject, e.g. \"CN=My Press\""))?;
            let dir = args.get(3).ok_or_else(|| anyhow::anyhow!("missing output folder"))?;
            let pw = password.ok_or_else(|| anyhow::anyhow!("pass --password or set BOOKCRAFT_CERT_PASSWORD"))?;
            let c = book_core::signing::create_test_certificate(subject, &pw, std::path::Path::new(dir))?;
            println!("Created {}\n        {}\nSubject {}, valid until {}", c.pfx, c.cer, c.info.subject, c.info.not_after);
        }
        Some("show") => {
            let file = args.get(2).ok_or_else(|| anyhow::anyhow!("missing .pfx path"))?;
            let i = book_core::signing::inspect(std::path::Path::new(file), password.as_deref())?;
            println!("{}", serde_json::to_string_pretty(&i)?);
        }
        _ => anyhow::bail!("usage: bookcraft cert new \"CN=Name\" <folder> --password <pw> | cert show <file.pfx>"),
    }
    Ok(())
}