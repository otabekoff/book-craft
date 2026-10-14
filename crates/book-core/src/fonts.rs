//! Fonts: Windows fonts every PC has, plus the bundled open-licence library
//! (built by scripts/fonts.mjs). Font keys are `georgia`, `segoe`, … for
//! Windows fonts and `lib:<id>` for bundled ones. A book only carries the
//! bundled fonts it actually uses.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::BookConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFont {
    pub id: String,
    pub name: String,
    /// The CSS family name used in its @font-face rules.
    pub family: String,
    /// `serif`, `sans`, `mono`, `display` or `handwriting`.
    pub category: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub subsets: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub kb: u64,
}

#[derive(Debug, Clone)]
pub struct FontLibrary {
    pub dir: PathBuf,
    pub fonts: Vec<LibraryFont>,
}

/// Windows fonts: key, name, category, stack.
const WINDOWS: &[(&str, &str, &str, &str)] = &[
    ("sitka", "Sitka Text", "serif", r#""Sitka Text", "Sitka Small", Georgia, serif"#),
    ("georgia", "Georgia", "serif", r#"Georgia, "Times New Roman", serif"#),
    ("cambria", "Cambria", "serif", "Cambria, Georgia, serif"),
    ("palatino", "Palatino Linotype", "serif", r#""Palatino Linotype", "Book Antiqua", Palatino, serif"#),
    ("garamond", "Garamond", "serif", r#"Garamond, Georgia, serif"#),
    ("constantia", "Constantia", "serif", "Constantia, Georgia, serif"),
    ("segoe", "Segoe UI", "sans", r#""Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif"#),
    ("verdana", "Verdana", "sans", "Verdana, Tahoma, sans-serif"),
    ("calibri", "Calibri", "sans", r#"Calibri, "Segoe UI", sans-serif"#),
    ("system-sans", "System sans-serif", "sans", "system-ui, sans-serif"),
    ("cascadia", "Cascadia Code", "mono", r#""Cascadia Code", "Cascadia Mono", Consolas, monospace"#),
    ("consolas", "Consolas", "mono", "Consolas, monospace"),
    ("custom", "Custom font file", "serif", r#""BookCustom", Georgia, serif"#),
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontOption {
    pub key: String,
    pub name: String,
    pub category: String,
    /// `library` (bundled, packed into the book) or `windows`.
    pub source: String,
    pub stack: String,
    pub note: String,
    /// Added to the book when used, in KB (0 for Windows fonts).
    pub kb: u64,
    /// Bundled fonts with only Latin glyphs fall back for Cyrillic text.
    pub cyrillic: bool,
}

fn fallback(category: &str) -> &'static str {
    match category {
        "sans" => r#""Segoe UI", system-ui, sans-serif"#,
        "mono" => r#""Cascadia Code", Consolas, monospace"#,
        "handwriting" => r#""Segoe Print", cursive"#,
        _ => "Georgia, serif",
    }
}

impl FontLibrary {
    pub fn load(dir: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(dir.join("fonts.json")).ok()?;
        let fonts = serde_json::from_str(&text).ok()?;
        Some(Self { dir: dir.to_path_buf(), fonts })
    }

    pub fn get(&self, id: &str) -> Option<&LibraryFont> {
        self.fonts.iter().find(|f| f.id == id)
    }

    /// @font-face rules for one font, with files under `base` (`…/_runtime/fonts/`).
    pub fn face_css(&self, id: &str, base: &str) -> String {
        std::fs::read_to_string(self.dir.join(id).join("font.css"))
            .map(|css| css.replace("__BASE__", &format!("{base}{id}/")))
            .unwrap_or_default()
    }

    /// The files to pack for one font (woff2 + licence).
    pub fn files(&self, id: &str) -> Vec<PathBuf> {
        let dir = self.dir.join(id);
        let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
        let mut out: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                name.ends_with(".woff2") || name == "LICENSE"
            })
            .collect();
        out.sort();
        out
    }
}

/// Every font the studio can offer, Windows fonts first.
pub fn options(lib: Option<&FontLibrary>) -> Vec<FontOption> {
    let mut out: Vec<FontOption> = WINDOWS
        .iter()
        .map(|(key, name, category, stack)| FontOption {
            key: key.to_string(),
            name: name.to_string(),
            category: category.to_string(),
            source: "windows".into(),
            stack: stack.to_string(),
            note: if *key == "custom" { "A .woff2/.ttf file from Assets".into() } else { "Built into Windows".into() },
            kb: 0,
            cyrillic: true,
        })
        .collect();
    if let Some(lib) = lib {
        for f in &lib.fonts {
            out.push(FontOption {
                key: format!("lib:{}", f.id),
                name: f.name.clone(),
                category: f.category.clone(),
                source: "library".into(),
                stack: format!("\"{}\", {}", f.family, fallback(&f.category)),
                note: f.note.clone(),
                kb: f.kb,
                cyrillic: f.subsets.iter().any(|s| s.starts_with("cyrillic")),
            });
        }
    }
    out
}

/// The CSS font-family stack for a font key.
pub fn stack(key: &str, lib: Option<&FontLibrary>) -> String {
    if let Some(id) = key.strip_prefix("lib:") {
        if let Some(f) = lib.and_then(|l| l.get(id)) {
            return format!("\"{}\", {}", f.family, fallback(&f.category));
        }
        // Library missing: fall back sensibly by guessing from the id.
        return fallback(if id.contains("mono") || id.contains("code") { "mono" } else { "serif" }).to_string();
    }
    WINDOWS.iter().find(|(k, ..)| *k == key).map(|(.., s)| s.to_string()).unwrap_or_else(|| WINDOWS[0].3.to_string())
}

pub fn display_name(key: &str, lib: Option<&FontLibrary>) -> String {
    if let Some(id) = key.strip_prefix("lib:") {
        return lib.and_then(|l| l.get(id)).map(|f| f.name.clone()).unwrap_or_else(|| id.to_string());
    }
    WINDOWS.iter().find(|(k, ..)| *k == key).map(|(_, n, ..)| n.to_string()).unwrap_or_else(|| key.to_string())
}

/// The keys of the typography roles; heading `same` means the body font.
pub fn role_keys(config: &BookConfig) -> [String; 4] {
    let t = &config.typography;
    let heading = if t.heading_font.is_empty() || t.heading_font == "same" { t.body_font.clone() } else { t.heading_font.clone() };
    [t.body_font.clone(), heading, t.ui_font.clone(), t.code_font.clone()]
}

/// Bundled fonts this book needs: its roles plus the ones offered to readers.
pub fn used_library_ids(config: &BookConfig) -> BTreeSet<String> {
    role_keys(config)
        .iter()
        .chain(config.reader.font_choices.iter())
        .filter_map(|k| k.strip_prefix("lib:").map(str::to_string))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_and_usage() {
        let lib = FontLibrary {
            dir: PathBuf::new(),
            fonts: vec![LibraryFont {
                id: "literata".into(),
                name: "Literata".into(),
                family: "Literata Variable".into(),
                category: "serif".into(),
                note: String::new(),
                italic: true,
                subsets: vec!["latin".into(), "cyrillic".into()],
                license: "OFL-1.1".into(),
                kb: 200,
            }],
        };
        assert!(stack("lib:literata", Some(&lib)).starts_with("\"Literata Variable\", Georgia"));
        assert!(stack("georgia", None).starts_with("Georgia"));
        let mut c = BookConfig::default();
        c.typography.body_font = "lib:literata".into();
        c.reader.font_choices = vec!["georgia".into(), "lib:inter".into()];
        let used = used_library_ids(&c);
        assert_eq!(used.into_iter().collect::<Vec<_>>(), vec!["inter", "literata"]);
        let opts = options(Some(&lib));
        assert!(opts.iter().any(|o| o.key == "lib:literata" && o.cyrillic && o.kb == 200));
    }
}
