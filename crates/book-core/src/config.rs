//! `book.json`: everything the author configures about a book.
//!
//! Every struct uses `#[serde(default)]` so old project files keep loading
//! after new options are added.

use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "book.json";
pub const TOML_CONFIG_FILE: &str = "book.toml";
pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BookConfig {
    pub version: u32,
    pub meta: Meta,
    pub chapters: Vec<ChapterRef>,
    pub typography: Typography,
    pub reader: ReaderSettings,
    pub modules: Modules,
    pub build: BuildSettings,
}

impl Default for BookConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            meta: Meta::default(),
            chapters: Vec::new(),
            typography: Typography::default(),
            reader: ReaderSettings::default(),
            modules: Modules::default(),
            build: BuildSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Meta {
    pub title: String,
    pub subtitle: String,
    pub author: String,
    pub publisher: String,
    /// Certificate subject used for MSIX identity, e.g. `CN=Harbourlight Press`.
    pub publisher_id: String,
    pub description: String,
    /// BCP-47 language tag, e.g. `en-US`.
    pub language: String,
    pub isbn: String,
    pub copyright: String,
    pub rights: String,
    pub website: String,
    /// Four-part numeric version, e.g. `1.0.0.0`.
    pub version: String,
    /// Reverse-domain identifier, e.g. `com.harbourlight.saltroad`.
    pub package_id: String,
    /// File name used for the generated `.exe` (without extension).
    pub executable_name: String,
    /// Path inside the project, e.g. `assets/icon.png`.
    pub icon: String,
    pub cover: String,
    pub accent: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            title: "Untitled Book".into(),
            subtitle: String::new(),
            author: String::new(),
            publisher: String::new(),
            publisher_id: String::new(),
            description: String::new(),
            language: "en-US".into(),
            isbn: String::new(),
            copyright: String::new(),
            rights: String::new(),
            website: String::new(),
            version: "1.0.0.0".into(),
            package_id: "com.example.book".into(),
            executable_name: String::new(),
            icon: String::new(),
            cover: String::new(),
            accent: "#2F5A8A".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChapterRef {
    /// Path relative to the project root, e.g. `chapters/01-intro.md`.
    pub file: String,
    /// Overrides the title taken from the chapter's first heading.
    pub title: String,
    /// Excluded chapters stay in the project but are not exported.
    pub hidden: bool,
    /// `chapter`, `part`, `front` (preface, dedication…) or `back` (appendix…).
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Typography {
    /// `sitka`, `georgia`, `cambria`, `palatino`, `segoe`, `system-sans` or `custom`.
    pub body_font: String,
    pub heading_font: String,
    /// Font file inside the project when `body_font` is `custom`.
    pub custom_font: String,
    pub base_size: u32,
    pub line_height: f32,
    pub max_width: u32,
    pub justify: bool,
    pub hyphenate: bool,
    pub drop_caps: bool,
    pub smart_quotes: bool,
    /// Text shown for `***` scene breaks. Empty = a plain rule.
    pub scene_break: String,
    pub paragraph_style: String, // `indent` | `spaced`
    pub heading_numbers: bool,
    /// Extra stylesheet inside the project, appended after the built-in styles.
    pub custom_css: String,
    /// Font for labels, captions, tables and the reader's own controls.
    pub ui_font: String,
    /// Font for code blocks and inline code.
    pub code_font: String,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            body_font: "sitka".into(),
            heading_font: "same".into(),
            custom_font: String::new(),
            base_size: 19,
            line_height: 1.65,
            max_width: 700,
            justify: false,
            hyphenate: true,
            drop_caps: false,
            smart_quotes: true,
            scene_break: "✻ ✻ ✻".into(),
            paragraph_style: "spaced".into(),
            heading_numbers: false,
            custom_css: String::new(),
            ui_font: "segoe".into(),
            code_font: "cascadia".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReaderSettings {
    /// `light`, `sepia`, `dark` or `system`.
    pub default_theme: String,
    pub allow_theme_change: bool,
    pub allow_font_size_change: bool,
    pub allow_font_change: bool,
    pub show_toc: bool,
    pub show_progress: bool,
    pub remember_position: bool,
    pub show_home_screen: bool,
    pub enable_search: bool,
    pub enable_bookmarks: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub allow_copy: bool,
    pub allow_print: bool,
    pub open_links_externally: bool,
    /// `custom` (the book's own title bar, matching its theme) or `native`.
    pub title_bar: String,
    /// Fonts readers may switch to (besides the book's own). Bundled ones are packed.
    pub font_choices: Vec<String>,
    /// How chapters open at first: `scroll` (one long page) or `pages` (turn pages like a book).
    pub default_layout: String,
    /// Pages per screen in page mode: `auto` (two when the window is wide), `one` or `two`.
    pub page_spread: String,
    /// Show the text on a paper sheet over a darker desk.
    pub paper_look: bool,
    /// How pages turn in page mode: `fade`, `slide`, `flip` (a page curl, like a real book) or `none`.
    pub page_turn: String,
    /// Show a number under each page in page mode.
    pub page_numbers: bool,
    /// Text column width readers start with: `narrow`, `normal`, `wide` or `full`.
    pub default_width: String,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            default_theme: "light".into(),
            allow_theme_change: true,
            allow_font_size_change: true,
            allow_font_change: true,
            show_toc: true,
            show_progress: true,
            remember_position: true,
            show_home_screen: true,
            enable_search: true,
            enable_bookmarks: true,
            window_width: 1200,
            window_height: 820,
            min_width: 480,
            min_height: 400,
            allow_copy: true,
            allow_print: true,
            open_links_externally: true,
            title_bar: "custom".into(),
            font_choices: vec!["georgia".into(), "segoe".into(), "verdana".into()],
            default_layout: "scroll".into(),
            page_spread: "auto".into(),
            paper_look: false,
            page_turn: "fade".into(),
            page_numbers: true,
            default_width: "normal".into(),
        }
    }
}

/// Per-block-type switches and options. A disabled module's syntax is left as
/// plain content (a directive becomes a neutral box, a fence a code block).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Modules {
    pub code: CodeModule,
    pub math: MathModule,
    pub diagrams: Toggle,
    pub charts: Toggle,
    pub canvas: Toggle,
    pub callouts: Toggle,
    pub quotes: QuoteModule,
    pub boxes: Toggle,
    pub layout: Toggle,
    pub media: Toggle,
    pub learning: Toggle,
    pub widgets: WidgetModule,
    pub footnotes: Toggle,
    pub raw_html: RawHtmlModule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Toggle {
    pub enabled: bool,
}

impl Default for Toggle {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CodeModule {
    pub enabled: bool,
    pub highlight: bool,
    pub line_numbers: bool,
    pub copy_button: bool,
    pub wrap_lines: bool,
    pub light_theme: String,
    pub dark_theme: String,
    /// Allow ```js {run=true}``` blocks to execute in a sandbox.
    pub runnable: bool,
}

impl Default for CodeModule {
    fn default() -> Self {
        Self {
            enabled: true,
            highlight: true,
            line_numbers: false,
            copy_button: true,
            wrap_lines: false,
            light_theme: "InspiredGitHub".into(),
            dark_theme: "OneHalfDark".into(),
            runnable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MathModule {
    pub enabled: bool,
    /// Number display equations that carry an id (`{#eq-x}`).
    pub number_equations: bool,
    /// Enables `\ce{}` and `\pu{}` (chemistry and units).
    pub chemistry: bool,
}

impl Default for MathModule {
    fn default() -> Self {
        Self { enabled: true, number_equations: true, chemistry: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct QuoteModule {
    pub enabled: bool,
    /// Style applied to plain `>` blockquotes.
    pub default_style: String,
}

impl Default for QuoteModule {
    fn default() -> Self {
        Self { enabled: true, default_style: "classic".into() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetModule {
    /// Custom HTML/JS widgets run in a sandboxed frame. Off by default.
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RawHtmlModule {
    /// Pass raw HTML in Markdown through unchanged. Off = it is shown as text.
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BuildSettings {
    pub portable: bool,
    pub nsis: bool,
    pub msi: bool,
    pub msix: bool,
    /// Relative to the project root.
    pub output_dir: String,
    /// `perUser` or `perMachine`.
    pub install_mode: String,
    pub desktop_shortcut: bool,
    pub start_menu_shortcut: bool,
    pub launch_after_install: bool,
    /// Plain-text license shown by the installers (project path). Optional.
    pub license_file: String,
    /// `embed` (bundle the bootstrapper), `download` or `skip`.
    pub webview2: String,
    pub signing: Signing,
    /// Display name in Add/Remove Programs; defaults to the title.
    pub product_name: String,
}

impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            portable: true,
            nsis: true,
            msi: false,
            msix: false,
            output_dir: "dist".into(),
            install_mode: "perUser".into(),
            desktop_shortcut: true,
            start_menu_shortcut: true,
            launch_after_install: true,
            license_file: String::new(),
            webview2: "embed".into(),
            signing: Signing::default(),
            product_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Signing {
    pub enabled: bool,
    /// Absolute path to a `.pfx` file. The password is never stored.
    pub certificate: String,
    pub timestamp_url: String,
}

impl Default for Signing {
    fn default() -> Self {
        Self {
            enabled: false,
            certificate: String::new(),
            timestamp_url: "http://timestamp.digicert.com".into(),
        }
    }
}

impl BookConfig {
    pub fn product_name(&self) -> String {
        let name = self.build.product_name.trim();
        if name.is_empty() { self.meta.title.trim().to_string() } else { name.to_string() }
    }

    /// A Windows-safe file stem for the executable.
    pub fn exe_stem(&self) -> String {
        let base = if self.meta.executable_name.trim().is_empty() {
            self.product_name()
        } else {
            self.meta.executable_name.trim().to_string()
        };
        let cleaned: String = base
            .chars()
            .map(|c| if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.') { c } else { '_' })
            .collect();
        let cleaned = cleaned.trim().trim_matches('.').to_string();
        if cleaned.is_empty() { "Book".into() } else { cleaned }
    }

    /// The version as four numbers, padding or truncating as needed.
    pub fn version_parts(&self) -> [u16; 4] {
        let mut out = [0u16; 4];
        for (i, part) in self.meta.version.split('.').take(4).enumerate() {
            out[i] = part.trim().parse().unwrap_or(0);
        }
        if out == [0; 4] {
            out[0] = 1;
        }
        out
    }

    pub fn version4(&self) -> String {
        let v = self.version_parts();
        format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3])
    }

    /// Checks the fields packaging depends on. Returns human-readable problems.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.meta.title.trim().is_empty() {
            problems.push("The book needs a title.".into());
        }
        let id = self.meta.package_id.trim();
        let id_ok = id.len() >= 3
            && id.len() <= 50
            && id.split('.').count() >= 2
            && id.split('.').all(|seg| {
                !seg.is_empty()
                    && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                    && seg.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            });
        if !id_ok {
            problems.push(
                "Package ID must look like com.publisher.book: 3–50 characters, letters, digits and hyphens, each part starting with a letter."
                    .into(),
            );
        }
        let parts: Vec<&str> = self.meta.version.split('.').collect();
        if parts.is_empty() || parts.len() > 4 || parts.iter().any(|p| p.trim().parse::<u16>().is_err()) {
            problems.push("Version must be up to four numbers, e.g. 1.0.0.0.".into());
        }
        if self.build.msi {
            let v = self.version_parts();
            if v[0] > 255 || v[1] > 255 {
                problems.push("MSI versions need the first two numbers to be 255 or less.".into());
            }
        }
        if self.build.msix && !self.meta.publisher_id.trim().starts_with("CN=") {
            problems.push(
                "MSIX needs a Publisher ID that matches your signing certificate, e.g. CN=Harbourlight Press.".into(),
            );
        }
        if self.chapters.iter().all(|c| c.hidden) {
            problems.push("Add at least one visible chapter.".into());
        }
        problems
    }
}
