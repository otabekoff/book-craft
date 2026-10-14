//! CSS generated from the book's typography and accent settings, plus the
//! runtime files every reader needs.

use std::path::Path;

use crate::config::BookConfig;
use crate::render::highlight;
use crate::samples::parse_hex;

pub const BLOCKS_CSS: &str = include_str!("../runtime/blocks.css");
pub const BLOCKS_JS: &str = include_str!("../runtime/blocks.js");

pub const FONTS: &[(&str, &str, &str)] = &[
    ("sitka", "Sitka (Windows reading font)", r#""Sitka Text", "Sitka Small", Georgia, serif"#),
    ("georgia", "Georgia", r#"Georgia, "Times New Roman", serif"#),
    ("cambria", "Cambria", "Cambria, Georgia, serif"),
    ("palatino", "Palatino", r#""Palatino Linotype", "Book Antiqua", Palatino, serif"#),
    ("garamond", "Garamond", r#"Garamond, "EB Garamond", Georgia, serif"#),
    ("constantia", "Constantia", "Constantia, Georgia, serif"),
    ("segoe", "Segoe UI", r#""Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif"#),
    ("verdana", "Verdana (large x-height)", "Verdana, Tahoma, sans-serif"),
    ("system-sans", "System sans-serif", "system-ui, sans-serif"),
    ("custom", "Custom font file", r#""BookCustom", Georgia, serif"#),
];

pub fn font_stack(key: &str) -> &'static str {
    FONTS.iter().find(|(k, _, _)| *k == key).map(|(_, _, s)| *s).unwrap_or(FONTS[0].2)
}

fn css_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace(['\n', '\r'], " "))
}

/// `:root` variables for fonts, sizes and colors. `asset_base` locates a custom font.
pub fn theme_css(config: &BookConfig, asset_base: &str) -> String {
    let t = &config.typography;
    let accent = parse_hex(&config.meta.accent).unwrap_or((0x2f, 0x5a, 0x8a));
    let hex = format!("#{:02x}{:02x}{:02x}", accent.0, accent.1, accent.2);
    // Pick readable text for filled accent surfaces.
    let lum = 0.2126 * srgb(accent.0) + 0.7152 * srgb(accent.1) + 0.0722 * srgb(accent.2);
    let on_accent = if lum > 0.45 { "#111111" } else { "#ffffff" };
    let body = font_stack(&t.body_font);
    let heading = if t.heading_font.is_empty() || t.heading_font == "same" { body } else { font_stack(&t.heading_font) };
    let mut css = String::new();
    if t.body_font == "custom" && !t.custom_font.is_empty() {
        css.push_str(&format!(
            "@font-face {{ font-family: \"BookCustom\"; src: url({}); font-display: swap; }}\n",
            css_string(&crate::render::asset_url_with(asset_base, &t.custom_font))
        ));
    }
    css.push_str(&format!(
        ":root {{ --bc-accent: {hex}; --bc-accent-ink: {}; --bc-on-accent: {on_accent}; --bc-font-body: {body}; --bc-font-heading: {heading}; --bc-size: {}px; --bc-leading: {}; --bc-measure: {}px; --bc-scene: {}; }}\n",
        if lum > 0.5 { "color-mix(in srgb, var(--bc-accent) 60%, black)" } else { "var(--bc-accent)" },
        t.base_size.clamp(12, 32),
        t.line_height.clamp(1.1, 2.4),
        t.max_width.clamp(420, 1400),
        if t.scene_break.trim().is_empty() { "\"\"".to_string() } else { css_string(&t.scene_break) },
    ));
    if t.scene_break.trim().is_empty() {
        css.push_str(".bc-chapter hr { border-top: 1px solid var(--bc-line); width: 30%; }\n");
    }
    css
}

fn srgb(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// Classes for the `<article>` that holds a chapter.
pub fn chapter_classes(config: &BookConfig) -> String {
    let t = &config.typography;
    let mut c = vec!["bc-chapter".to_string()];
    if t.drop_caps {
        c.push("bc-dropcaps".into());
    }
    if t.justify {
        c.push("bc-justify".into());
    }
    if t.hyphenate {
        c.push("bc-hyphenate".into());
    }
    if t.paragraph_style == "indent" {
        c.push("bc-indent".into());
    }
    if t.heading_numbers {
        c.push("bc-heading-numbers".into());
    }
    let q = crate::render::safe_token(&config.modules.quotes.default_style);
    c.push(format!("bc-quotes-{q}"));
    c.join(" ")
}

pub fn syntax_css(config: &BookConfig) -> String {
    highlight::theme_css(&config.modules.code.light_theme, &config.modules.code.dark_theme)
}

/// The project's custom stylesheet, if any.
pub fn custom_css(config: &BookConfig, project_root: &Path) -> String {
    if config.typography.custom_css.trim().is_empty() {
        return String::new();
    }
    crate::project::resolve_inside(project_root, &config.typography.custom_css)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default()
}

/// All CSS for a book in load order: blocks, syntax, theme, custom.
pub fn book_css(config: &BookConfig, project_root: &Path, asset_base: &str) -> String {
    format!(
        "{}\n/* syntax */\n{}\n/* theme */\n{}\n/* custom */\n{}",
        BLOCKS_CSS,
        syntax_css(config),
        theme_css(config, asset_base),
        custom_css(config, project_root)
    )
}
