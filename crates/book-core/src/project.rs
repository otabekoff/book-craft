//! A book project on disk:
//!
//! ```text
//! my-book/
//!   book.json        settings, metadata, chapter order
//!   chapters/*.md    one Markdown file per chapter
//!   assets/**        images, audio, video, fonts, icon, cover
//!   dist/            build output (ignored by the project)
//! ```

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::config::{BookConfig, CONFIG_FILE, ChapterRef, TOML_CONFIG_FILE};
use crate::samples;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterInfo {
    pub file: String,
    pub title: String,
    pub hidden: bool,
    pub kind: String,
    pub words: usize,
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub path: String,
    pub size: u64,
    pub kind: String,
}

/// Which file holds the book's settings. Both carry the same data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFormat {
    Json,
    Toml,
}

impl ConfigFormat {
    pub fn file_name(self) -> &'static str {
        match self {
            ConfigFormat::Json => CONFIG_FILE,
            ConfigFormat::Toml => TOML_CONFIG_FILE,
        }
    }
}

pub struct Project {
    pub root: PathBuf,
    pub config: BookConfig,
    pub format: ConfigFormat,
}

impl Project {
    /// Opens a book folder. `book.toml` wins if both settings files exist.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let format = if root.join(TOML_CONFIG_FILE).is_file() {
            ConfigFormat::Toml
        } else if root.join(CONFIG_FILE).is_file() {
            ConfigFormat::Json
        } else {
            bail!("{} is not a book folder (no {TOML_CONFIG_FILE} or {CONFIG_FILE})", root.display());
        };
        let name = format.file_name();
        let text = fs::read_to_string(root.join(name)).with_context(|| format!("Reading {name}"))?;
        let config: BookConfig = match format {
            ConfigFormat::Json => serde_json::from_str(&text).with_context(|| format!("{name} is not valid"))?,
            ConfigFormat::Toml => toml::from_str(&text).with_context(|| format!("{name} is not valid"))?,
        };
        Ok(Self { root, config, format })
    }

    pub fn is_book_folder(root: &Path) -> bool {
        root.join(CONFIG_FILE).is_file() || root.join(TOML_CONFIG_FILE).is_file()
    }

    /// Switches the settings file to `format`, removing the old one.
    pub fn set_format(&mut self, format: ConfigFormat) -> Result<()> {
        if format == self.format {
            return Ok(());
        }
        let old = self.root.join(self.format.file_name());
        self.format = format;
        self.save()?;
        if old.is_file() {
            fs::remove_file(old)?;
        }
        Ok(())
    }

    /// Creates a new project with sample chapters that show every block type.
    pub fn create(root: impl AsRef<Path>, title: &str, author: &str) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        if Self::is_book_folder(&root) {
            bail!("{} already contains a book project", root.display());
        }
        fs::create_dir_all(root.join("chapters"))?;
        fs::create_dir_all(root.join("assets"))?;

        let mut config = BookConfig::default();
        config.meta.title = title.trim().to_string();
        config.meta.author = author.trim().to_string();
        config.meta.copyright = format!("© {} {}", current_year(), author.trim());
        config.meta.package_id = format!("com.bookcraft.{}", slug(title).replace('-', ""));
        if config.meta.package_id.len() > 50 || slug(title).is_empty() {
            config.meta.package_id = "com.bookcraft.book".into();
        }

        for (file, body) in samples::chapters(title) {
            fs::write(root.join(&file), body)?;
            config.chapters.push(ChapterRef { file, kind: "chapter".into(), ..Default::default() });
        }
        fs::write(root.join("assets/icon.png"), samples::default_icon_png(&config.meta.accent)?)?;
        config.meta.icon = "assets/icon.png".into();

        let project = Self { root, config, format: ConfigFormat::Json };
        project.save()?;
        fs::write(project.root.join(".gitignore"), "dist/\n")?;
        Ok(project)
    }

    pub fn save(&self) -> Result<()> {
        let text = match self.format {
            ConfigFormat::Json => serde_json::to_string_pretty(&self.config)?,
            ConfigFormat::Toml => format!(
                "# Book Craft settings. Edit here or in the studio.\n\n{}",
                toml::to_string_pretty(&self.config)?
            ),
        };
        let name = self.format.file_name();
        let tmp = self.root.join(format!("{name}.tmp"));
        fs::write(&tmp, text)?;
        fs::rename(&tmp, self.root.join(name))?;
        Ok(())
    }

    /// Resolves a project-relative path, refusing anything that escapes the project.
    pub fn resolve(&self, relative: &str) -> Result<PathBuf> {
        resolve_inside(&self.root, relative)
    }

    pub fn chapters(&self) -> Vec<ChapterInfo> {
        self.config
            .chapters
            .iter()
            .map(|c| {
                let text = self.resolve(&c.file).ok().and_then(|p| fs::read_to_string(p).ok());
                let missing = text.is_none();
                let text = text.unwrap_or_default();
                ChapterInfo {
                    file: c.file.clone(),
                    title: chapter_title(c, &text),
                    hidden: c.hidden,
                    kind: if c.kind.is_empty() { "chapter".into() } else { c.kind.clone() },
                    words: text.split_whitespace().count(),
                    missing,
                }
            })
            .collect()
    }

    pub fn read_chapter(&self, file: &str) -> Result<String> {
        Ok(fs::read_to_string(self.resolve(file)?)?)
    }

    pub fn write_chapter(&self, file: &str, content: &str) -> Result<()> {
        let path = self.resolve(file)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Adds a new chapter file after `after` (or at the end) and returns its path.
    pub fn add_chapter(&mut self, title: &str, after: Option<&str>) -> Result<String> {
        let stem = {
            let s = slug(title);
            if s.is_empty() { "chapter".to_string() } else { s }
        };
        let mut n = self.config.chapters.len() + 1;
        let file = loop {
            let candidate = format!("chapters/{n:02}-{stem}.md");
            if !self.root.join(&candidate).exists() {
                break candidate;
            }
            n += 1;
        };
        let heading = if title.trim().is_empty() { "New chapter" } else { title.trim() };
        self.write_chapter(&file, &format!("# {heading}\n\n"))?;
        let entry = ChapterRef { file: file.clone(), kind: "chapter".into(), ..Default::default() };
        let index = after
            .and_then(|a| self.config.chapters.iter().position(|c| c.file == a))
            .map(|i| i + 1)
            .unwrap_or(self.config.chapters.len());
        self.config.chapters.insert(index, entry);
        self.save()?;
        Ok(file)
    }

    /// Removes the chapter from the book. The file is moved to `chapters/.trash`.
    pub fn remove_chapter(&mut self, file: &str) -> Result<()> {
        let before = self.config.chapters.len();
        self.config.chapters.retain(|c| c.file != file);
        if self.config.chapters.len() == before {
            bail!("{file} is not part of this book");
        }
        let path = self.resolve(file)?;
        if path.exists() {
            let trash = self.root.join("chapters/.trash");
            fs::create_dir_all(&trash)?;
            let name = path.file_name().ok_or_else(|| anyhow!("bad file name"))?;
            fs::rename(&path, trash.join(name))?;
        }
        self.save()
    }

    pub fn assets(&self) -> Result<Vec<AssetInfo>> {
        let dir = self.root.join("assets");
        let mut out = Vec::new();
        if !dir.exists() {
            return Ok(out);
        }
        for entry in walkdir::WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry.path().strip_prefix(&self.root)?.to_string_lossy().replace('\\', "/");
            out.push(AssetInfo {
                kind: asset_kind(&rel).into(),
                size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                path: rel,
            });
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }

    /// Copies files into `assets/<subdir>` and returns their project paths.
    pub fn import_assets(&self, sources: &[PathBuf], subdir: &str) -> Result<Vec<String>> {
        let target_dir = self.resolve(&format!("assets/{}", subdir.trim_matches('/')))?;
        fs::create_dir_all(&target_dir)?;
        let mut out = Vec::new();
        for src in sources {
            let name = src
                .file_name()
                .ok_or_else(|| anyhow!("{} has no file name", src.display()))?
                .to_string_lossy()
                .to_string();
            let safe: String = name
                .chars()
                .map(|c| if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '-' })
                .collect();
            let mut dest = target_dir.join(&safe);
            let mut i = 2;
            while dest.exists() {
                let p = Path::new(&safe);
                let stem = p.file_stem().unwrap_or_default().to_string_lossy();
                let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
                dest = target_dir.join(format!("{stem}-{i}{ext}"));
                i += 1;
            }
            fs::copy(src, &dest).with_context(|| format!("Copying {}", src.display()))?;
            out.push(dest.strip_prefix(&self.root)?.to_string_lossy().replace('\\', "/"));
        }
        Ok(out)
    }

    pub fn delete_asset(&self, path: &str) -> Result<()> {
        if !path.starts_with("assets/") {
            bail!("Only files in assets/ can be deleted");
        }
        fs::remove_file(self.resolve(path)?)?;
        Ok(())
    }

    pub fn output_dir(&self) -> Result<PathBuf> {
        let dir = if self.config.build.output_dir.trim().is_empty() {
            "dist"
        } else {
            self.config.build.output_dir.trim()
        };
        let path = Path::new(dir);
        if path.is_absolute() { Ok(path.to_path_buf()) } else { self.resolve(dir) }
    }
}

pub fn resolve_inside(root: &Path, relative: &str) -> Result<PathBuf> {
    let rel = Path::new(relative.trim_start_matches(['/', '\\']));
    let mut out = root.to_path_buf();
    for component in rel.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => bail!("Path {relative:?} points outside the project"),
        }
    }
    Ok(out)
}

pub fn chapter_title(entry: &ChapterRef, markdown: &str) -> String {
    if !entry.title.trim().is_empty() {
        return entry.title.trim().to_string();
    }
    let mut in_fence = false;
    for line in markdown.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && t.starts_with("# ") {
            // Drop a trailing `{#id .class}` attribute block.
            let title = t[2..].trim();
            let title = match title.rfind(" {") {
                Some(i) if title.ends_with('}') => &title[..i],
                _ => title,
            };
            return title.trim().to_string();
        }
    }
    Path::new(&entry.file)
        .file_stem()
        .map(|s| s.to_string_lossy().replace(['-', '_'], " "))
        .unwrap_or_else(|| "Untitled".into())
}

pub fn asset_kind(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" | "bmp" | "ico" => "image",
        "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" => "audio",
        "mp4" | "webm" | "mov" | "m4v" => "video",
        "woff" | "woff2" | "ttf" | "otf" => "font",
        "pdf" => "pdf",
        "glb" | "gltf" => "model",
        _ => "file",
    }
}

pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in text.trim().chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_end_matches('-').chars().take(40).collect()
}

fn current_year() -> i32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Good enough for a copyright line: 365.2425-day years since 1970.
    1970 + (secs as f64 / 31_556_952.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_rejects_escape() {
        let root = Path::new("C:/book");
        assert!(resolve_inside(root, "../x").is_err());
        assert!(resolve_inside(root, "assets/../../x").is_err());
        assert!(resolve_inside(root, "C:/Windows").is_err());
        assert_eq!(resolve_inside(root, "assets/a.png").unwrap(), root.join("assets").join("a.png"));
    }

    #[test]
    fn titles_from_heading() {
        let c = ChapterRef { file: "chapters/01-x.md".into(), ..Default::default() };
        assert_eq!(chapter_title(&c, "```\n# no\n```\n# Real {#id}\n"), "Real");
        assert_eq!(chapter_title(&c, "text"), "01 x");
    }

    #[test]
    fn creates_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let mut p = Project::create(dir.path(), "The Salt Road", "Ann Author").unwrap();
        assert!(p.config.validate().is_empty(), "{:?}", p.config.validate());
        let f = p.add_chapter("Epilogue", None).unwrap();
        assert!(p.read_chapter(&f).unwrap().starts_with("# Epilogue"));
        let again = Project::open(dir.path()).unwrap();
        assert_eq!(again.config.chapters.len(), p.config.chapters.len());
        p.remove_chapter(&f).unwrap();
        assert!(!dir.path().join(&f).exists());
    }

    #[test]
    fn switches_between_json_and_toml() {
        let dir = tempfile::tempdir().unwrap();
        let mut p = Project::create(dir.path(), "Formats", "A").unwrap();
        p.config.meta.subtitle = "with \"quotes\" and ünïcode".into();
        p.set_format(ConfigFormat::Toml).unwrap();
        assert!(dir.path().join("book.toml").is_file());
        assert!(!dir.path().join("book.json").exists());
        let t = Project::open(dir.path()).unwrap();
        assert_eq!(t.format, ConfigFormat::Toml);
        assert_eq!(t.config.meta.subtitle, p.config.meta.subtitle);
        assert_eq!(t.config.chapters.len(), p.config.chapters.len());
        assert_eq!(t.config.typography.line_height, p.config.typography.line_height);
        let mut t = t;
        t.set_format(ConfigFormat::Json).unwrap();
        assert!(Project::open(dir.path()).unwrap().format == ConfigFormat::Json);
    }
}
