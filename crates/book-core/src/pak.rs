//! Builds `book.pak`: the zip archive the reader app loads.
//!
//! ```text
//! book.json                  manifest: metadata, reader settings, chapters, TOC
//! chapters/001.html …        rendered chapters
//! assets/**                  the project's assets, unchanged
//! _app/icon.png              window icon
//! _runtime/book.css          block styles + code theme + book theme + custom CSS
//! _runtime/blocks.js         block behaviours
//! _runtime/vendor/**         KaTeX / Mermaid, only when the book uses them
//! ```

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::config::{BookConfig, ReaderSettings};
use crate::project::{Project, chapter_title};
use crate::render::{Heading, RenderOptions, render_markdown};
use crate::theme;

pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: u32,
    pub generator: String,
    pub built_at: u64,
    pub meta: PublicMeta,
    pub reader: ReaderSettings,
    pub chapter_class: String,
    pub chapters: Vec<ManifestChapter>,
    pub features: BTreeSet<String>,
    pub words: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicMeta {
    pub title: String,
    pub subtitle: String,
    pub author: String,
    pub publisher: String,
    pub description: String,
    pub language: String,
    pub isbn: String,
    pub copyright: String,
    pub rights: String,
    pub website: String,
    pub version: String,
    pub package_id: String,
    pub accent: String,
    pub cover: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestChapter {
    pub id: String,
    pub file: String,
    pub source: String,
    pub title: String,
    pub kind: String,
    pub headings: Vec<Heading>,
    pub words: usize,
}

pub struct Pak {
    pub size: u64,
    pub manifest: Manifest,
    pub warnings: Vec<String>,
}

fn is_compressed(path: &str) -> bool {
    let ext = Path::new(path).extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "mp3" | "mp4" | "m4a" | "m4v" | "ogg" | "webm" | "mov"
            | "aac" | "flac" | "woff" | "woff2" | "zip" | "pdf" | "glb"
    )
}

struct Writer {
    zip: ZipWriter<BufWriter<File>>,
}

impl Writer {
    fn add(&mut self, path: &str, data: &[u8]) -> Result<()> {
        let method = if is_compressed(path) { CompressionMethod::Stored } else { CompressionMethod::Deflated };
        let opts = SimpleFileOptions::default().compression_method(method).large_file(data.len() > u32::MAX as usize / 2);
        self.zip.start_file(path, opts)?;
        self.zip.write_all(data)?;
        Ok(())
    }
}

pub fn public_meta(c: &BookConfig) -> PublicMeta {
    let m = &c.meta;
    PublicMeta {
        title: m.title.clone(),
        subtitle: m.subtitle.clone(),
        author: m.author.clone(),
        publisher: m.publisher.clone(),
        description: m.description.clone(),
        language: m.language.clone(),
        isbn: m.isbn.clone(),
        copyright: m.copyright.clone(),
        rights: m.rights.clone(),
        website: m.website.clone(),
        version: c.version4(),
        package_id: m.package_id.clone(),
        accent: m.accent.clone(),
        cover: m.cover.clone(),
    }
}

/// Renders every visible chapter and packs the book.
/// `vendor_dir` holds `katex/` and `mermaid.min.js`; `icon_png` is the window icon.
pub fn build(
    project: &Project,
    vendor_dir: Option<&Path>,
    icon_png: &[u8],
    out: &Path,
    log: &mut dyn FnMut(&str),
) -> Result<Pak> {
    let config = &project.config;
    let file = File::create(out).with_context(|| format!("Creating {}", out.display()))?;
    let mut w = Writer { zip: ZipWriter::new(BufWriter::new(file)) };
    let mut warnings = Vec::new();
    let mut features = BTreeSet::new();
    let mut chapters = Vec::new();
    let mut total_words = 0;

    let visible: Vec<_> = config.chapters.iter().filter(|c| !c.hidden).collect();
    for (i, entry) in visible.iter().enumerate() {
        let md = project
            .read_chapter(&entry.file)
            .with_context(|| format!("Reading chapter {}", entry.file))?;
        let opts = RenderOptions { config, asset_base: "/".into(), chapter_file: Some(entry.file.clone()) };
        let r = render_markdown(&md, &opts);
        for wmsg in &r.warnings {
            warnings.push(format!("{}: {wmsg}", entry.file));
        }
        features.extend(r.features.iter().cloned());
        total_words += r.words;
        let file = format!("chapters/{:03}.html", i + 1);
        w.add(&file, r.html.as_bytes())?;
        chapters.push(ManifestChapter {
            id: format!("c{}", i + 1),
            file,
            source: entry.file.clone(),
            title: if entry.title.trim().is_empty() {
                r.title.clone().unwrap_or_else(|| chapter_title(entry, &md))
            } else {
                entry.title.trim().to_string()
            },
            kind: if entry.kind.is_empty() { "chapter".into() } else { entry.kind.clone() },
            headings: r.headings.into_iter().filter(|h| h.level >= 2 && h.level <= 3).collect(),
            words: r.words,
        });
        log(&format!("Rendered {}", entry.file));
    }

    // Assets: everything under assets/.
    let assets = project.root.join("assets");
    let mut asset_count = 0;
    if assets.exists() {
        for e in walkdir::WalkDir::new(&assets).into_iter().filter_map(|e| e.ok()) {
            if !e.file_type().is_file() {
                continue;
            }
            let rel = e.path().strip_prefix(&project.root)?.to_string_lossy().replace('\\', "/");
            let data = std::fs::read(e.path()).with_context(|| format!("Reading {rel}"))?;
            w.add(&rel, &data)?;
            asset_count += 1;
        }
    }
    log(&format!("Packed {asset_count} asset file(s)"));

    w.add("_app/icon.png", icon_png)?;
    w.add("_runtime/book.css", theme::book_css(config, &project.root, "/").as_bytes())?;
    w.add("_runtime/blocks.js", theme::BLOCKS_JS.as_bytes())?;

    let needs_math = features.contains("math");
    let needs_mermaid = features.contains("mermaid");
    match vendor_dir {
        Some(dir) if dir.exists() => {
            if needs_math {
                let katex = dir.join("katex");
                if katex.exists() {
                    for e in walkdir::WalkDir::new(&katex).into_iter().filter_map(|e| e.ok()) {
                        if e.file_type().is_file() {
                            let rel = e.path().strip_prefix(dir)?.to_string_lossy().replace('\\', "/");
                            w.add(&format!("_runtime/vendor/{rel}"), &std::fs::read(e.path())?)?;
                        }
                    }
                    log("Included math typesetting (KaTeX)");
                } else {
                    warnings.push("Math is used but KaTeX was not found; formulas will show as source.".into());
                }
            }
            if needs_mermaid {
                let m = dir.join("mermaid.min.js");
                if m.exists() {
                    w.add("_runtime/vendor/mermaid.min.js", &std::fs::read(m)?)?;
                    log("Included diagrams (Mermaid)");
                } else {
                    warnings.push("Diagrams are used but Mermaid was not found; they will show as text.".into());
                }
            }
        }
        _ if needs_math || needs_mermaid => {
            warnings.push("Math or diagram libraries were not found; those blocks will show their source.".into());
        }
        _ => {}
    }

    let manifest = Manifest {
        format: FORMAT_VERSION,
        generator: format!("Book Craft {}", env!("CARGO_PKG_VERSION")),
        built_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        meta: public_meta(config),
        reader: config.reader.clone(),
        chapter_class: theme::chapter_classes(config),
        chapters,
        features,
        words: total_words,
    };
    w.add("book.json", serde_json::to_string_pretty(&manifest)?.as_bytes())?;
    w.zip.finish()?.flush()?;
    let size = std::fs::metadata(out)?.len();
    Ok(Pak { size, manifest, warnings })
}
