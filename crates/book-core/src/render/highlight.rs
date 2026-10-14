//! Syntax highlighting at export time with class-based spans, so light and
//! dark themes are plain CSS and the reader ships no highlighter.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, css_for_theme_with_class_style, line_tokens_to_classed_spans};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

use super::escape;

const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "hl-" };

fn syntaxes() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(two_face::syntax::extra_newlines)
}

fn themes() -> &'static ThemeSet {
    static SET: OnceLock<ThemeSet> = OnceLock::new();
    SET.get_or_init(|| ThemeSet::from(two_face::theme::extra()))
}

/// Theme names offered in the studio.
pub fn theme_names() -> Vec<String> {
    let skip = ["ansi", "base16", "base16-256"];
    themes().themes.keys().filter(|k| !skip.contains(&k.as_str())).cloned().collect()
}

/// Language tokens offered in the studio's code-block picker.
pub fn language_names() -> Vec<String> {
    let mut out = BTreeSet::new();
    for s in syntaxes().syntaxes() {
        if s.hidden {
            continue;
        }
        if let Some(ext) = s.file_extensions.first() {
            out.insert(format!("{}|{}", s.name, ext));
        }
    }
    out.into_iter().collect()
}

fn find_syntax(lang: &str) -> Option<&'static SyntaxReference> {
    let set = syntaxes();
    let lang = lang.trim().to_ascii_lowercase();
    if lang.is_empty() {
        return None;
    }
    let alias = match lang.as_str() {
        "js" | "javascript" | "jsx" | "mjs" => "js",
        "ts" | "typescript" => "ts",
        "tsx" => "tsx",
        "py" | "python" => "py",
        "rs" | "rust" => "rs",
        "sh" | "bash" | "shell" | "zsh" => "sh",
        "ps" | "ps1" | "powershell" | "pwsh" => "ps1",
        "c#" | "cs" | "csharp" => "cs",
        "c++" | "cpp" | "cxx" => "cpp",
        "kt" | "kotlin" => "kt",
        "yml" | "yaml" => "yaml",
        "md" | "markdown" => "md",
        "golang" | "go" => "go",
        "rb" | "ruby" => "rb",
        "html" | "htm" | "xhtml" => "html",
        "vue" => "vue",
        "dockerfile" | "docker" => "Dockerfile",
        "jsonc" | "json5" => "json",
        "tex" | "latex" => "tex",
        "asm" | "nasm" | "x86asm" => "asm",
        "patch" | "diff" => "diff",
        "sql" | "mysql" | "postgres" | "postgresql" => "sql",
        other => other,
    };
    set.find_syntax_by_token(alias)
        .or_else(|| set.find_syntax_by_extension(alias))
        .or_else(|| set.find_syntax_by_name(&lang))
        .or_else(|| set.syntaxes().iter().find(|s| s.name.eq_ignore_ascii_case(&lang)))
}

/// Returns one HTML string per source line (spans balanced within each line).
/// Falls back to escaped plain text when the language is unknown.
pub fn highlight_lines(code: &str, lang: &str, enabled: bool) -> Vec<String> {
    let lines: Vec<&str> = code.split_inclusive('\n').collect();
    let syntax = if enabled { find_syntax(lang) } else { None };
    let Some(syntax) = syntax else {
        return lines.iter().map(|l| escape(l.trim_end_matches(['\n', '\r']))).collect();
    };

    let set = syntaxes();
    let mut state = ParseState::new(syntax);
    let mut scopes = ScopeStack::new();
    // Opening tags still unclosed at the end of the previous line.
    let mut carried: Vec<String> = Vec::new();
    let mut out = Vec::with_capacity(lines.len());

    for line in lines {
        let ops = match state.parse_line(line, set) {
            Ok(ops) => ops,
            Err(_) => {
                out.push(escape(line.trim_end_matches(['\n', '\r'])));
                continue;
            }
        };
        let (html, _) = match line_tokens_to_classed_spans(line, &ops, CLASS_STYLE, &mut scopes) {
            Ok(r) => r,
            Err(_) => {
                out.push(escape(line.trim_end_matches(['\n', '\r'])));
                continue;
            }
        };
        let mut rendered: String = carried.concat();
        rendered.push_str(&html);
        // Track which spans are still open so each line stands alone.
        let mut stack = carried.clone();
        let mut rest = html.as_str();
        while let Some(pos) = rest.find('<') {
            rest = &rest[pos..];
            if rest.starts_with("</span>") {
                stack.pop();
                rest = &rest[7..];
            } else if rest.starts_with("<span") {
                let end = rest.find('>').map(|e| e + 1).unwrap_or(rest.len());
                stack.push(rest[..end].to_string());
                rest = &rest[end..];
            } else {
                rest = &rest[1..];
            }
        }
        for _ in 0..stack.len() {
            rendered.push_str("</span>");
        }
        carried = stack;
        let trimmed = rendered.replace('\n', "").replace('\r', "");
        out.push(trimmed);
    }
    out
}

/// CSS for both code themes; the dark theme applies under `[data-theme=dark]`.
pub fn theme_css(light: &str, dark: &str) -> String {
    let set = themes();
    let pick = |name: &str, fallback: &str| {
        set.themes.get(name).or_else(|| set.themes.get(fallback)).cloned()
    };
    let mut css = String::new();
    if let Some(theme) = pick(light, "InspiredGitHub") {
        if let Ok(c) = css_for_theme_with_class_style(&theme, CLASS_STYLE) {
            css.push_str(&scope_css(&c, ":root"));
            if let Some(bg) = theme.settings.background {
                css.push_str(&format!(
                    ":root {{ --bc-code-bg: rgb({} {} {}); }}\n",
                    bg.r, bg.g, bg.b
                ));
            }
        }
    }
    if let Some(theme) = pick(dark, "OneHalfDark") {
        if let Ok(c) = css_for_theme_with_class_style(&theme, CLASS_STYLE) {
            css.push_str(&scope_css(&c, "[data-theme=dark]"));
            if let Some(bg) = theme.settings.background {
                css.push_str(&format!(
                    "[data-theme=dark] {{ --bc-code-bg: rgb({} {} {}); }}\n",
                    bg.r, bg.g, bg.b
                ));
            }
        }
    }
    css
}

/// Prefixes every selector and drops syntect's `.code` background rule,
/// which the book's own code block styles replace.
fn scope_css(css: &str, scope: &str) -> String {
    let mut out = String::new();
    for block in css.split('}') {
        let Some((selectors, body)) = block.split_once('{') else { continue };
        let body = body.trim();
        let mut sels = Vec::new();
        for sel in selectors.split(',') {
            // Remove comments that syntect places before rules.
            let sel = match sel.rfind("*/") {
                Some(i) => &sel[i + 2..],
                None => sel,
            };
            let sel = sel.trim();
            if sel.is_empty() || sel == ".hl-code" {
                continue;
            }
            sels.push(format!("{scope} {sel}"));
        }
        if !sels.is_empty() && !body.is_empty() {
            // Code text sits on the book's own background; drop syntect backgrounds.
            let body: Vec<&str> = body
                .split(';')
                .map(str::trim)
                .filter(|d| !d.is_empty() && !d.starts_with("background-color"))
                .collect();
            if !body.is_empty() {
                out.push_str(&format!("{} {{ {}; }}\n", sels.join(", "), body.join("; ")));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiline_comment_stays_balanced() {
        let lines = highlight_lines("let a = 1; /* one\ntwo */ let b = 2;\n", "rust", true);
        assert_eq!(lines.len(), 2);
        for l in &lines {
            assert_eq!(l.matches("<span").count(), l.matches("</span>").count(), "{l}");
        }
        assert!(lines[0].contains("hl-"));
    }

    #[test]
    fn unknown_language_is_escaped() {
        assert_eq!(highlight_lines("<b>\n", "nope-lang", true), vec!["&lt;b&gt;"]);
    }

    #[test]
    fn css_scopes() {
        let css = theme_css("InspiredGitHub", "OneHalfDark");
        assert!(css.contains("[data-theme=dark] .hl-"));
        assert!(css.contains("--bc-code-bg"));
    }
}
