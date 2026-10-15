//! Markdown → HTML for one chapter, with the book's block modules applied.

pub mod attrs;
mod blocks;
pub mod canvas;
pub mod chart;
pub mod highlight;

use std::collections::{BTreeSet, HashSet, VecDeque};
use std::fmt;
use std::sync::Mutex;

use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::{AstNode, NodeValue, Sourcepos};
use comrak::options::Plugins;
use comrak::{Arena, Options, format_html_with_plugins, parse_document};
use serde::Serialize;

use crate::config::BookConfig;

pub use blocks::safe_token;

pub struct RenderOptions<'a> {
    pub config: &'a BookConfig,
    /// Prefix for relative asset URLs (`assets/x.png`). `/` in the reader.
    pub asset_base: String,
    /// The chapter's own project path, used to resolve links to other chapters.
    pub chapter_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Rendered {
    pub html: String,
    pub title: Option<String>,
    pub headings: Vec<Heading>,
    pub features: BTreeSet<String>,
    pub warnings: Vec<String>,
    pub words: usize,
}

pub(crate) struct Ctx<'o, 'c> {
    pub opts: &'o RenderOptions<'c>,
    pub features: BTreeSet<String>,
    pub warnings: Vec<String>,
}

impl Ctx<'_, '_> {
    pub fn feature(&mut self, f: &str) {
        self.features.insert(f.to_string());
    }

    pub fn warn(&mut self, w: &str) {
        if !self.warnings.iter().any(|x| x == w) {
            self.warnings.push(w.to_string());
        }
    }

    pub fn asset_url(&self, src: &str) -> String {
        asset_url_with(&self.opts.asset_base, src)
    }

    pub fn error_block(&mut self, what: &str, err: &str, source: &str) -> String {
        self.warn(&format!("{what}: {err}"));
        format!(
            r#"<div class="bc-error" role="note"><strong>{} could not be drawn:</strong> {}<pre>{}</pre></div>"#,
            escape(what),
            escape(err),
            escape(source)
        ) + "\n"
    }
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

fn is_relative(url: &str) -> bool {
    !(url.is_empty()
        || url.starts_with('#')
        || url.starts_with('/')
        || url.starts_with("data:")
        || url.starts_with("mailto:")
        || url.contains("://"))
}

pub fn asset_url_with(base: &str, src: &str) -> String {
    let src = src.trim();
    if !is_relative(src) {
        return src.to_string();
    }
    let src = src.trim_start_matches("./");
    // Strip `../` that would leave the book; chapters live one level down.
    let src = src.trim_start_matches("../");
    format!("{base}{src}")
}

/// Resolves `other.md#part` relative to the current chapter into a project path.
fn chapter_link(current: Option<&str>, url: &str) -> Option<String> {
    let (path, frag) = match url.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (url, None),
    };
    if !path.to_ascii_lowercase().ends_with(".md") || !is_relative(path) {
        return None;
    }
    let dir = current.and_then(|c| c.rsplit_once('/').map(|(d, _)| d)).unwrap_or("");
    let mut parts: Vec<&str> = if dir.is_empty() { Vec::new() } else { dir.split('/').collect() };
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    let mut target = format!("#chapter:{}", parts.join("/"));
    if let Some(f) = frag {
        target.push('#');
        target.push_str(f);
    }
    Some(target)
}

/// Assigns stable, unique ids to headings and records them for the table of contents.
struct Headings {
    custom_ids: Mutex<VecDeque<Option<String>>>,
    used: Mutex<HashSet<String>>,
    found: Mutex<Vec<Heading>>,
}

impl HeadingAdapter for Headings {
    fn enter(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta, _: Option<Sourcepos>) -> fmt::Result {
        let custom = self.custom_ids.lock().unwrap().pop_front().flatten();
        let mut used = self.used.lock().unwrap();
        let base = custom.unwrap_or_else(|| {
            let s = crate::project::slug(&heading.content);
            if s.is_empty() { "section".into() } else { s }
        });
        let mut id = base.clone();
        let mut n = 1;
        while used.contains(&id) {
            id = format!("{base}-{n}");
            n += 1;
        }
        used.insert(id.clone());
        self.found.lock().unwrap().push(Heading {
            level: heading.level,
            id: id.clone(),
            text: heading.content.trim().to_string(),
        });
        write!(output, r#"<h{} id="{}">"#, heading.level, escape(&id))
    }

    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        writeln!(output, "</h{}>", heading.level)
    }
}

pub fn comrak_options(config: &BookConfig) -> Options<'static> {
    let m = &config.modules;
    let mut o = Options::default();
    o.extension.strikethrough = true;
    o.extension.table = true;
    o.extension.autolink = true;
    o.extension.tasklist = true;
    o.extension.superscript = true;
    o.extension.subscript = true;
    o.extension.footnotes = m.footnotes.enabled;
    o.extension.description_lists = true;
    o.extension.front_matter_delimiter = Some("---".into());
    o.extension.alerts = m.callouts.enabled;
    o.extension.math_dollars = m.math.enabled;
    o.extension.math_code = m.math.enabled;
    o.extension.highlight = true;
    o.extension.spoiler = true;
    o.extension.block_directive = true;
    o.extension.header_attributes = true;
    o.parse.smart = config.typography.smart_quotes;
    o.render.r#unsafe = m.raw_html.enabled;
    o.render.tasklist_classes = true;
    o.render.figure_with_caption = true;
    o
}

pub fn render_markdown(markdown: &str, opts: &RenderOptions) -> Rendered {
    let options = comrak_options(opts.config);
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &options);
    let mut ctx = Ctx { opts, features: BTreeSet::new(), warnings: Vec::new() };
    let raw_allowed = opts.config.modules.raw_html.enabled;

    let nodes: Vec<&AstNode> = root.descendants().collect();
    let mut custom_ids = VecDeque::new();
    let mut words = 0usize;

    // Pass 1: inline fixes and data collection.
    for node in &nodes {
        let mut data = node.data_mut();
        let attrs_id = data.attrs.as_ref().and_then(|a| a.id.clone());
        match &mut data.value {
            NodeValue::Heading(_) => custom_ids.push_back(attrs_id),
            NodeValue::Text(t) => words += t.split_whitespace().count(),
            NodeValue::HtmlBlock(b) if !raw_allowed => {
                let text = b.literal.clone();
                data.value = NodeValue::Paragraph;
                drop(data);
                node.append(arena.alloc(AstNode::from(NodeValue::Text(text.into()))));
            }
            NodeValue::HtmlInline(h) if !raw_allowed => {
                let text = std::mem::take(h);
                data.value = NodeValue::Text(text.into());
            }
            NodeValue::Image(link) => link.url = ctx.asset_url(&link.url),
            NodeValue::Link(link) => {
                if let Some(target) = chapter_link(opts.chapter_file.as_deref(), &link.url) {
                    link.url = target;
                } else if is_relative(&link.url) && link.url.starts_with("assets/") {
                    link.url = asset_url_with(&opts.asset_base, &link.url);
                }
            }
            NodeValue::Math(_) => ctx.feature("math"),
            NodeValue::Alert(_) => ctx.feature("callout"),
            NodeValue::FootnoteDefinition(_) => ctx.feature("footnotes"),
            NodeValue::Table(_) => ctx.feature("table"),
            _ => {}
        }
    }

    // Pass 2: fenced code becomes block HTML.
    for node in &nodes {
        let block = match &node.data().value {
            NodeValue::CodeBlock(cb) => Some((cb.info.clone(), cb.literal.clone())),
            _ => None,
        };
        if let Some((info, literal)) = block {
            let html = blocks::code_block(&info, &literal, &mut ctx);
            node.data_mut().value = NodeValue::Raw(html);
        }
    }

    // Pass 3: directives, innermost first, are replaced by open/children/close.
    for node in nodes.iter().rev() {
        let info = match &node.data().value {
            NodeValue::BlockDirective(d) => Some(d.info.clone()),
            _ => None,
        };
        let Some(info) = info else { continue };
        let (open, close) = blocks::directive(&info, &mut ctx);
        node.insert_before(arena.alloc(AstNode::from(NodeValue::Raw(open))));
        let children: Vec<&AstNode> = node.children().collect();
        for child in children {
            child.detach();
            node.insert_before(child);
        }
        node.insert_before(arena.alloc(AstNode::from(NodeValue::Raw(close))));
        node.detach();
    }

    let headings = Headings {
        custom_ids: Mutex::new(custom_ids),
        used: Mutex::new(HashSet::new()),
        found: Mutex::new(Vec::new()),
    };
    let mut plugins = Plugins::default();
    plugins.render.heading_adapter = Some(&headings);
    let mut html = String::new();
    if format_html_with_plugins(root, &options, &mut html, &plugins).is_err() {
        ctx.warn("The chapter could not be rendered completely.");
    }
    let html = html
        .replace("<p><figure>", "<figure class=\"bc-image\">")
        .replace("</figure></p>", "</figure>")
        .replace("<img src=", "<img loading=\"lazy\" src=");

    let found = headings.found.into_inner().unwrap();
    let title = found.iter().find(|h| h.level == 1).map(|h| h.text.clone());
    Rendered {
        html,
        title,
        headings: found,
        features: ctx.features,
        warnings: ctx.warnings,
        words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(md: &str) -> Rendered {
        let config = BookConfig::default();
        let opts = RenderOptions { config: &config, asset_base: "/".into(), chapter_file: Some("chapters/01.md".into()) };
        render_markdown(md, &opts)
    }

    #[test]
    fn headings_get_ids() {
        let r = render("# Hello World\n\n## Part {#custom}\n\n## Part\n\n## Part\n");
        assert_eq!(r.title.as_deref(), Some("Hello World"));
        let ids: Vec<_> = r.headings.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, ["hello-world", "custom", "part", "part-1"]);
        assert!(r.html.contains(r#"<h2 id="custom">Part</h2>"#), "{}", r.html);
    }

    #[test]
    fn callout_directive_wraps_markdown() {
        let r = render(":::: tip {title=\"Remember\"}\nSome **bold** text.\n\n::: box {style=shaded}\nInner\n:::\n::::\n");
        assert!(r.html.contains("bc-callout bc-callout-tip"), "{}", r.html);
        assert!(r.html.contains("<strong>bold</strong>"));
        assert!(r.html.contains("bc-box bc-box-shaded"));
        let open = r.html.find("bc-callout-tip").unwrap();
        let inner = r.html.find("bc-box-shaded").unwrap();
        assert!(inner > open);
    }

    #[test]
    fn collapsible_blocks_start_collapsed() {
        let closed = render("::: note {collapsible=true}\nHidden\n:::\n\n::: details\nMore\n:::\n");
        assert!(closed.html.contains("<details class=\"bc-callout"), "{}", closed.html);
        assert!(!closed.html.contains(" open>"), "{}", closed.html);
        let open = render("::: note {collapsible=true collapsed=false}\nShown\n:::\n\n::: details {open}\nMore\n:::\n");
        assert_eq!(open.html.matches(" open>").count(), 2, "{}", open.html);
    }

    #[test]
    fn quote_with_attribution() {
        let r = render("::: quote {style=pull author=\"Ada Lovelace\" source=\"Notes\"}\nImagination is the discovering faculty.\n:::\n");
        assert!(r.html.contains("bc-quote bc-quote-pull"));
        assert!(r.html.contains("Ada Lovelace"));
        assert!(r.html.contains("<cite>Notes</cite>"));
    }

    #[test]
    fn code_is_highlighted_with_lines() {
        let r = render("```rust {title=\"main.rs\" lines=true highlight=2}\nfn main() {\n    println!(\"hi\");\n}\n```\n");
        assert!(r.html.contains("bc-code-numbered"), "{}", r.html);
        assert!(r.html.contains("bc-line bc-line-mark\" data-n=\"2\""));
        assert!(r.html.contains("main.rs"));
        assert!(r.html.contains("hl-"));
        assert!(r.features.contains("code"));
    }

    #[test]
    fn math_inline_and_block() {
        let r = render("Energy $E=mc^2$.\n\n```math {#eq-e}\nE = mc^2\n```\n");
        assert!(r.features.contains("math"));
        assert!(r.html.contains(r#"data-math-style="inline""#));
        assert!(r.html.contains("bc-eq-numbered"));
    }

    #[test]
    fn raw_html_is_text_by_default() {
        let r = render("<script>alert(1)</script>\n\nHi <b>x</b>\n");
        assert!(!r.html.contains("<script>"), "{}", r.html);
        assert!(r.html.contains("&lt;script&gt;"));
        assert!(!r.html.contains("<b>"));
    }

    #[test]
    fn links_and_images() {
        let r = render("![Cat](../assets/cat.png \"A cat\")\n\n[Next](02-next.md#start) [Web](https://x.org)\n");
        assert!(r.html.contains(r#"src="/assets/cat.png""#), "{}", r.html);
        assert!(r.html.contains("<figcaption>A cat</figcaption>"));
        assert!(r.html.contains("#chapter:chapters/02-next.md#start"));
        assert!(r.html.contains("https://x.org"));
    }

    #[test]
    fn widgets_off_by_default() {
        let r = render("```widget\n<script>1</script>\n```\n");
        assert!(!r.html.contains("<iframe"));
        assert!(!r.warnings.is_empty());
    }

    #[test]
    fn chart_canvas_mermaid() {
        let r = render("```chart {type=line}\nx,a\n1,2\n2,3\n```\n\n```canvas\ncircle 10 10 5\n```\n\n```mermaid\ngraph TD; A-->B\n```\n");
        assert!(r.features.contains("chart"));
        assert!(r.features.contains("canvas"));
        assert!(r.features.contains("mermaid"));
        assert!(r.html.contains("A--&gt;B"));
    }

    #[test]
    fn learning_blocks() {
        let md = "::: quiz\nWhat is 2+2?\n\n- [ ] 3\n- [x] 4\n\n> Because maths.\n:::\n\n::: exercise {title=\"Sum\"}\nAdd them.\n\n::: solution\nIt is 4.\n:::\n:::\n";
        let r = render(md);
        assert!(r.html.contains("bc-quiz"));
        assert!(r.html.contains("task-list-item"), "{}", r.html);
        assert!(r.html.contains("<details class=\"bc-solution"));
    }
}
