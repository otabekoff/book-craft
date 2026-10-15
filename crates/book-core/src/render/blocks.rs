//! HTML for `:::` directives and special fenced code blocks.

use super::attrs::{self, Info};
use super::highlight::highlight_lines;
use super::{Ctx, canvas, chart, escape};

// ---------------------------------------------------------------- directives

const CALLOUTS: &[(&str, &str)] = &[
    ("note", "Note"),
    ("tip", "Tip"),
    ("info", "Info"),
    ("warning", "Warning"),
    ("caution", "Caution"),
    ("danger", "Danger"),
    ("important", "Important"),
    ("success", "Success"),
    ("question", "Question"),
    ("example", "Example"),
    ("definition", "Definition"),
    ("theorem", "Theorem"),
    ("lemma", "Lemma"),
    ("corollary", "Corollary"),
    ("proposition", "Proposition"),
    ("proof", "Proof"),
    ("remark", "Remark"),
    ("summary", "Summary"),
    ("keypoints", "Key points"),
    ("objectives", "Learning objectives"),
    ("history", "Historical note"),
];

/// Numbered callouts get "Theorem 3" style titles via CSS counters.
const NUMBERED: &[&str] = &["theorem", "lemma", "corollary", "proposition", "definition", "example"];

/// Collapsible blocks (collapsible callouts, details, spoilers, hints, solutions) start
/// collapsed unless the author writes `collapsed=false` or `open`.
fn starts_open(info: &Info) -> bool {
    info.flag("open") || !info.flag_or("collapsed", true)
}

/// Returns the opening and closing HTML that wrap a directive's rendered children.
pub fn directive(raw_info: &str, ctx: &mut Ctx) -> (String, String) {
    let info = attrs::parse(raw_info);
    let m = &ctx.opts.config.modules;
    let name = info.name.as_str();
    let id = info.id.as_ref().map(|i| format!(r#" id="{}""#, escape(i))).unwrap_or_default();
    let extra = classes(&info);
    // `::: Callout title text` — words after the name become the title.
    let title_text = info.get("title").map(str::to_string).or_else(|| {
        if info.rest.is_empty() { None } else { Some(info.rest.clone()) }
    });

    // Callouts (also `::: callout {type=…}`).
    let callout_type = if name == "callout" || name == "admonition" {
        Some(info.get_or("type", "note").to_ascii_lowercase())
    } else if CALLOUTS.iter().any(|(n, _)| *n == name) {
        Some(name.to_string())
    } else {
        None
    };
    if let Some(kind) = callout_type {
        if m.callouts.enabled {
            ctx.feature("callout");
            let label = CALLOUTS.iter().find(|(n, _)| *n == kind).map(|(_, l)| *l).unwrap_or("Note");
            let numbered = NUMBERED.contains(&kind.as_str()) && !info.flag("unnumbered");
            let title_html = match &title_text {
                Some(t) if numbered => format!(
                    r#"<span class="bc-callout-label">{label}</span><span class="bc-callout-name">{}</span>"#,
                    escape(t)
                ),
                Some(t) => format!(r#"<span class="bc-callout-name">{}</span>"#, escape(t)),
                None => format!(r#"<span class="bc-callout-label">{label}</span>"#),
            };
            let cls = format!(
                "bc-callout bc-callout-{}{}{}",
                escape(&kind),
                if numbered { " bc-numbered" } else { "" },
                extra
            );
            let style = color_style(&info);
            // Collapsible callouts start collapsed; `collapsed=false` (or `open`) starts them open.
            if info.flag("collapsible") || info.get("collapsed").is_some() {
                let open = if starts_open(&info) { " open" } else { "" };
                return (
                    format!(
                        r#"<details class="{cls}"{id}{style}{open}><summary class="bc-callout-title"><span class="bc-callout-icon" aria-hidden="true"></span>{title_html}</summary><div class="bc-callout-body">"#
                    ),
                    "</div></details>\n".into(),
                );
            }
            return (
                format!(
                    r#"<aside class="{cls}"{id}{style}><div class="bc-callout-title"><span class="bc-callout-icon" aria-hidden="true"></span>{title_html}</div><div class="bc-callout-body">"#
                ),
                "</div></aside>\n".into(),
            );
        }
    }

    match name {
        "box" | "frame" | "card" | "shape" if m.boxes.enabled => {
            ctx.feature("box");
            let style = safe_token(info.get_or("style", if name == "card" { "card" } else { "framed" }));
            let shape = safe_token(info.get_or("shape", "rounded"));
            let mut css = Vec::new();
            if let Some(c) = info.get("color").and_then(css_color) {
                css.push(format!("--bc-box-color:{c}"));
            }
            if let Some(c) = info.get("background").and_then(css_color) {
                css.push(format!("--bc-box-bg:{c}"));
            }
            if let Some(w) = info.get("width").and_then(css_length) {
                css.push(format!("max-width:{w}"));
            }
            if let Some(a) = info.get("align") {
                css.push(format!("text-align:{}", safe_align(a)));
            }
            if info.get("float").is_some_and(|f| f == "right" || f == "left") {
                css.push(format!("float:{}", info.get("float").unwrap_or("right")));
            }
            let style_attr = if css.is_empty() { String::new() } else { format!(r#" style="{}""#, css.join(";")) };
            let title = title_text
                .map(|t| format!(r#"<div class="bc-box-title">{}</div>"#, escape(&t)))
                .unwrap_or_default();
            (
                format!(
                    r#"<div class="bc-box bc-box-{style} bc-shape-{shape}{extra}"{id}{style_attr}>{title}<div class="bc-box-body">"#
                ),
                "</div></div>\n".into(),
            )
        }
        "quote" | "epigraph" | "pullquote" | "pull-quote" | "testimonial" if m.quotes.enabled => {
            ctx.feature("quote");
            let style = match name {
                "epigraph" => "epigraph".to_string(),
                "pullquote" | "pull-quote" => "pull".to_string(),
                "testimonial" => "card".to_string(),
                _ => safe_token(info.get_or("style", &ctx.opts.config.modules.quotes.default_style)),
            };
            let mut cite = String::new();
            if let Some(a) = info.get("author") {
                cite.push_str(&format!(r#"<span class="bc-quote-author">{}</span>"#, escape(a)));
            }
            if let Some(s) = info.get("source") {
                if !cite.is_empty() {
                    cite.push_str(", ");
                }
                cite.push_str(&format!("<cite>{}</cite>", escape(s)));
            }
            if let Some(r) = info.get("role") {
                cite.push_str(&format!(r#"<span class="bc-quote-role">{}</span>"#, escape(r)));
            }
            let avatar = info
                .get("image")
                .map(|src| format!(r#"<img class="bc-quote-avatar" src="{}" alt="">"#, escape(&ctx.asset_url(src))))
                .unwrap_or_default();
            let caption = if cite.is_empty() && avatar.is_empty() {
                String::new()
            } else {
                format!(r#"<figcaption>{avatar}<span>{cite}</span></figcaption>"#)
            };
            (
                format!(r#"<figure class="bc-quote bc-quote-{style}{extra}"{id}{}><blockquote>"#, color_style(&info)),
                format!("</blockquote>{caption}</figure>\n"),
            )
        }
        "verse" | "poem" | "lyrics" => {
            let title = title_text
                .map(|t| format!(r#"<div class="bc-verse-title">{}</div>"#, escape(&t)))
                .unwrap_or_default();
            let author = info
                .get("author")
                .map(|a| format!(r#"<div class="bc-verse-author">{}</div>"#, escape(a)))
                .unwrap_or_default();
            (
                format!(r#"<div class="bc-verse{extra}"{id}>{title}<div class="bc-verse-body">"#),
                format!("</div>{author}</div>\n"),
            )
        }
        "dialogue" | "dialog" | "script" | "screenplay" => {
            (format!(r#"<div class="bc-dialogue{extra}"{id}>"#), "</div>\n".into())
        }
        "letter" | "note-card" => {
            (format!(r#"<div class="bc-letter{extra}"{id}>"#), "</div>\n".into())
        }
        "aside" | "sidebar" | "sidenote" | "margin" => {
            let title = title_text
                .map(|t| format!(r#"<div class="bc-aside-title">{}</div>"#, escape(&t)))
                .unwrap_or_default();
            (format!(r#"<aside class="bc-aside bc-aside-{name}{extra}"{id}>{title}"#), "</aside>\n".into())
        }
        "center" | "right" | "left" | "justify" => {
            (format!(r#"<div class="bc-align" style="text-align:{name}"{id}>"#), "</div>\n".into())
        }
        "lead" | "large" | "small" | "smallcaps" | "nodropcap" | "dropcap" => {
            (format!(r#"<div class="bc-{name}{extra}"{id}>"#), "</div>\n".into())
        }
        "columns" | "cols" if m.layout.enabled => {
            let n = info.number("count").or_else(|| info.number("cols")).unwrap_or(2.0).clamp(1.0, 6.0) as u32;
            let gap = info.get("gap").and_then(css_length).map(|g| format!(";--bc-gap:{g}")).unwrap_or_default();
            let mode = if info.flag("flow") { " bc-columns-flow" } else { "" };
            (
                format!(r#"<div class="bc-columns{mode}{extra}"{id} style="--bc-cols:{n}{gap}">"#),
                "</div>\n".into(),
            )
        }
        "column" | "col" if m.layout.enabled => {
            let span = info.number("span").unwrap_or(1.0).clamp(1.0, 6.0) as u32;
            (format!(r#"<div class="bc-column{extra}"{id} style="grid-column:span {span}">"#), "</div>\n".into())
        }
        "tabs" if m.layout.enabled => {
            ctx.feature("tabs");
            (format!(r#"<div class="bc-tabs{extra}"{id}>"#), "</div>\n".into())
        }
        "tab" if m.layout.enabled => {
            let title = title_text.unwrap_or_else(|| "Tab".into());
            (
                format!(r#"<section class="bc-tab{extra}"{id} data-title="{}">"#, escape(&title)),
                "</section>\n".into(),
            )
        }
        "details" | "spoiler" | "collapse" | "accordion" => {
            let summary = title_text
                .or_else(|| info.get("summary").map(str::to_string))
                .unwrap_or_else(|| if name == "spoiler" { "Show spoiler".into() } else { "Details".into() });
            let open = if starts_open(&info) { " open" } else { "" };
            (
                format!(
                    r#"<details class="bc-details bc-details-{name}{extra}"{id}{open}><summary>{}</summary><div class="bc-details-body">"#,
                    escape(&summary)
                ),
                "</div></details>\n".into(),
            )
        }
        "figure" if m.media.enabled => {
            ctx.feature("figure");
            let mut open = format!(
                r#"<figure class="bc-figure{}{extra}"{id}{}>"#,
                if info.flag_or("numbered", true) { " bc-numbered" } else { "" },
                info.get("width").and_then(css_length).map(|w| format!(r#" style="max-width:{w}""#)).unwrap_or_default()
            );
            if let Some(src) = info.get("src") {
                let alt = info.get("alt").or(info.get("caption")).unwrap_or("");
                open.push_str(&format!(
                    r#"<img src="{}" alt="{}" loading="lazy">"#,
                    escape(&ctx.asset_url(src)),
                    escape(alt)
                ));
            }
            let mut cap = String::new();
            if let Some(c) = info.get("caption").or(title_text.as_deref()) {
                cap.push_str(&escape(c));
            }
            if let Some(c) = info.get("credit") {
                cap.push_str(&format!(r#" <span class="bc-credit">{}</span>"#, escape(c)));
            }
            let close = if cap.is_empty() {
                "</figure>\n".to_string()
            } else {
                format!("<figcaption>{cap}</figcaption></figure>\n")
            };
            (format!("{open}<div class=\"bc-figure-body\">"), format!("</div>{close}"))
        }
        "gallery" if m.media.enabled => {
            ctx.feature("gallery");
            let n = info.number("columns").unwrap_or(3.0).clamp(1.0, 6.0) as u32;
            (format!(r#"<div class="bc-gallery{extra}"{id} style="--bc-cols:{n}">"#), "</div>\n".into())
        }
        "compare" | "before-after" if m.media.enabled => {
            ctx.feature("compare");
            let before = info.get("before").map(|s| ctx.asset_url(s)).unwrap_or_default();
            let after = info.get("after").map(|s| ctx.asset_url(s)).unwrap_or_default();
            (
                format!(
                    r#"<figure class="bc-compare{extra}"{id}><div class="bc-compare-frame" data-before="{}" data-after="{}"><img src="{}" alt="Before"><div class="bc-compare-top"><img src="{}" alt="After"></div><input class="bc-compare-range" type="range" min="0" max="100" value="50" aria-label="Compare before and after"></div><figcaption>"#,
                    escape(&before),
                    escape(&after),
                    escape(&before),
                    escape(&after)
                ),
                "</figcaption></figure>\n".into(),
            )
        }
        "audio" | "video" if m.media.enabled => {
            ctx.feature(name);
            let src = info.get("src").map(|s| ctx.asset_url(s)).unwrap_or_default();
            let title = title_text
                .map(|t| format!(r#"<div class="bc-media-title">{}</div>"#, escape(&t)))
                .unwrap_or_default();
            let el = if name == "audio" {
                format!(r#"<audio controls preload="metadata" src="{}"></audio>"#, escape(&src))
            } else {
                let poster = info
                    .get("poster")
                    .map(|p| format!(r#" poster="{}""#, escape(&ctx.asset_url(p))))
                    .unwrap_or_default();
                let flags = format!(
                    "{}{}{}",
                    if info.flag("loop") { " loop" } else { "" },
                    if info.flag("muted") || info.flag("autoplay") { " muted" } else { "" },
                    if info.flag("autoplay") { " autoplay" } else { "" }
                );
                format!(r#"<video controls preload="metadata" src="{}"{poster}{flags}></video>"#, escape(&src))
            };
            (
                format!(r#"<figure class="bc-media bc-{name}{extra}"{id}>{title}{el}<figcaption>"#),
                "</figcaption></figure>\n".into(),
            )
        }
        "attachment" | "download" | "pdf" if m.media.enabled => {
            ctx.feature("attachment");
            let src = info.get("src").map(|s| ctx.asset_url(s)).unwrap_or_default();
            let label = title_text.or_else(|| info.get("label").map(str::to_string)).unwrap_or_else(|| "Open file".into());
            (
                format!(
                    r#"<div class="bc-attachment{extra}"{id}><a class="bc-attachment-link" href="{}" data-viewer="1"><span class="bc-attachment-icon" aria-hidden="true"></span><span>{}</span></a><div class="bc-attachment-body">"#,
                    escape(&src),
                    escape(&label)
                ),
                "</div></div>\n".into(),
            )
        }
        "quiz" if m.learning.enabled => {
            ctx.feature("quiz");
            let title = title_text.unwrap_or_else(|| "Quick check".into());
            (
                format!(
                    r#"<section class="bc-quiz{extra}"{id} data-multiple="{}"><div class="bc-quiz-title">{}</div><div class="bc-quiz-body">"#,
                    info.flag("multiple"),
                    escape(&title)
                ),
                "</div></section>\n".into(),
            )
        }
        "flashcards" | "cards" if m.learning.enabled => {
            ctx.feature("flashcards");
            (format!(r#"<div class="bc-flashcards{extra}"{id}>"#), "</div>\n".into())
        }
        "exercise" | "problem" | "task" if m.learning.enabled => {
            ctx.feature("exercise");
            let title = title_text
                .map(|t| format!(r#"<span class="bc-exercise-name">{}</span>"#, escape(&t)))
                .unwrap_or_default();
            let diff = info
                .get("difficulty")
                .map(|d| format!(r#"<span class="bc-difficulty" data-level="{}">{}</span>"#, safe_token(d), escape(d)))
                .unwrap_or_default();
            (
                format!(
                    r#"<section class="bc-exercise{extra}"{id}><div class="bc-exercise-title"><span class="bc-exercise-label">Exercise</span>{title}{diff}</div><div class="bc-exercise-body">"#
                ),
                "</div></section>\n".into(),
            )
        }
        "solution" | "answer" | "hint" if m.learning.enabled => {
            let label = title_text.unwrap_or_else(|| match name {
                "hint" => "Show hint".into(),
                "answer" => "Show answer".into(),
                _ => "Show solution".into(),
            });
            let open = if starts_open(&info) { " open" } else { "" };
            (
                format!(
                    r#"<details class="bc-solution bc-{name}{extra}"{id}{open}><summary>{}</summary><div class="bc-solution-body">"#,
                    escape(&label)
                ),
                "</div></details>\n".into(),
            )
        }
        "steps" => (format!(r#"<div class="bc-steps{extra}"{id}>"#), "</div>\n".into()),
        "timeline" => (format!(r#"<div class="bc-timeline{extra}"{id}>"#), "</div>\n".into()),
        "glossary" | "definitions" => (format!(r#"<div class="bc-glossary{extra}"{id}>"#), "</div>\n".into()),
        "pagebreak" | "page-break" => ("<div class=\"bc-pagebreak\" aria-hidden=\"true\">".into(), "</div>\n".into()),
        "chapter-opener" | "opener" => {
            let label = info
                .get("label")
                .map(|l| format!(r#"<div class="bc-opener-label">{}</div>"#, escape(l)))
                .unwrap_or_default();
            let image = info
                .get("image")
                .map(|src| format!(r#"<img class="bc-opener-image" src="{}" alt="">"#, escape(&ctx.asset_url(src))))
                .unwrap_or_default();
            (format!(r#"<header class="bc-opener{extra}"{id}>{image}{label}"#), "</header>\n".into())
        }
        _ => {
            // Unknown or disabled: a neutral container authors can style with custom CSS.
            let title = title_text
                .map(|t| format!(r#"<div class="bc-directive-title">{}</div>"#, escape(&t)))
                .unwrap_or_default();
            (
                format!(r#"<div class="bc-directive bc-{}{extra}"{id}>{title}"#, safe_token(name)),
                "</div>\n".into(),
            )
        }
    }
}

fn classes(info: &Info) -> String {
    info.classes.iter().map(|c| format!(" {}", safe_token(c))).collect()
}

fn color_style(info: &Info) -> String {
    match info.get("color").and_then(css_color) {
        Some(c) => format!(r#" style="--bc-tone:{c}""#),
        None => String::new(),
    }
}

/// Keeps only characters that are safe in a class name.
pub fn safe_token(s: &str) -> String {
    let t: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_ascii_lowercase();
    if t.is_empty() { "default".into() } else { t }
}

fn safe_align(s: &str) -> &'static str {
    match s {
        "center" => "center",
        "right" | "end" => "right",
        "justify" => "justify",
        _ => "left",
    }
}

/// Accepts hex colors or theme color names; anything else is rejected.
pub fn css_color(v: &str) -> Option<String> {
    canvas::color(v).ok()
}

fn css_length(v: &str) -> Option<String> {
    let v = v.trim();
    let (num, unit) = v.split_at(v.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(v.len()));
    let n: f64 = num.parse().ok()?;
    let unit = match unit {
        "" | "px" => "px",
        "%" => "%",
        "em" => "em",
        "rem" => "rem",
        "ch" => "ch",
        _ => return None,
    };
    Some(format!("{n}{unit}"))
}

// ---------------------------------------------------------------- code fences

pub fn code_block(raw_info: &str, literal: &str, ctx: &mut Ctx) -> String {
    let info = attrs::parse(raw_info);
    let m = &ctx.opts.config.modules;
    let code = literal.strip_suffix('\n').unwrap_or(literal);
    let caption = info.get("caption").map(str::to_string);
    let id = info.id.as_ref().map(|i| format!(r#" id="{}""#, escape(i))).unwrap_or_default();

    match info.name.as_str() {
        "math" | "equation" | "latex-math" if m.math.enabled => {
            ctx.feature("math");
            math_display(code, &info, ctx)
        }
        "chem" | "ce" | "chemistry" if m.math.enabled && m.math.chemistry => {
            ctx.feature("math");
            ctx.feature("chemistry");
            let tex: Vec<String> = code
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| format!(r"\ce{{{}}}", l.trim()))
                .collect();
            math_display(&tex.join(r" \\ "), &info, ctx)
        }
        "mermaid" if m.diagrams.enabled => {
            ctx.feature("mermaid");
            let cap = caption.map(|c| format!("<figcaption>{}</figcaption>", escape(&c))).unwrap_or_default();
            format!(r#"<figure class="bc-mermaid"{id}><pre class="mermaid">{}</pre>{cap}</figure>"#, escape(code)) + "\n"
        }
        "chart" | "plot" if m.charts.enabled => match chart::render(code, &info) {
            Ok(svg) => {
                ctx.feature("chart");
                svg + "\n"
            }
            Err(e) => ctx.error_block("Chart", &e, code),
        },
        "canvas" | "draw" | "shapes" | "svgdraw" if m.canvas.enabled => {
            let title = caption.clone().or_else(|| info.get("title").map(str::to_string));
            let base = ctx.opts.asset_base.clone();
            let resolver = move |s: &str| super::asset_url_with(&base, s);
            match canvas::render(code, title.as_deref(), &resolver) {
                Ok(svg) => {
                    ctx.feature("canvas");
                    svg + "\n"
                }
                Err(e) => ctx.error_block("Canvas", &e, code),
            }
        }
        "widget" | "html-widget" | "embed" => {
            if !m.widgets.enabled {
                ctx.warn("A widget block was skipped because custom widgets are turned off (Modules → Widgets).");
                return code_html(code, "html", &info, ctx);
            }
            ctx.feature("widget");
            let height = info.number("height").unwrap_or(320.0).clamp(60.0, 2000.0);
            let title = info.get("title").unwrap_or("Interactive widget");
            let cap = caption.map(|c| format!("<figcaption>{}</figcaption>", escape(&c))).unwrap_or_default();
            format!(
                r#"<figure class="bc-widget"{id}><iframe class="bc-widget-frame" sandbox="allow-scripts" referrerpolicy="no-referrer" loading="lazy" title="{}" style="height:{height}px" srcdoc="{}"></iframe>{cap}</figure>"#,
                escape(title),
                escape(code)
            ) + "\n"
        }
        "terminal" | "console" | "shell-session" | "cmd" | "powershell-session" => {
            ctx.feature("code");
            terminal(code, &info, ctx)
        }
        lang => {
            let lang = lang.to_string();
            code_html(code, &lang, &info, ctx)
        }
    }
}

fn math_display(tex: &str, info: &Info, ctx: &mut Ctx) -> String {
    let numbered = ctx.opts.config.modules.math.number_equations && (info.id.is_some() || info.flag("numbered"));
    let id = info.id.as_ref().map(|i| format!(r#" id="{}""#, escape(i))).unwrap_or_default();
    format!(
        r#"<div class="bc-math-block{}"{id}><div class="bc-math" data-math-style="display">{}</div>{}</div>"#,
        if numbered { " bc-eq-numbered" } else { "" },
        escape(tex),
        if numbered { r#"<span class="bc-eq-num" aria-hidden="true"></span>"# } else { "" }
    ) + "\n"
}

fn code_html(code: &str, lang: &str, info: &Info, ctx: &mut Ctx) -> String {
    let cm = &ctx.opts.config.modules.code;
    ctx.feature("code");
    let numbered = info.flag_or("lines", cm.line_numbers) || info.get("start").is_some();
    let start = info.number("start").unwrap_or(1.0).max(0.0) as usize;
    let marked = info.get("highlight").or(info.get("hl")).map(attrs::line_ranges).unwrap_or_default();
    let wrap = info.flag_or("wrap", cm.wrap_lines);
    let is_diff = lang == "diff" || lang == "patch" || info.flag("diff");
    let runnable = cm.runnable && info.flag("run") && matches!(lang, "js" | "javascript" | "mjs");
    if runnable {
        ctx.feature("run");
    }

    let lines = highlight_lines(code, lang, cm.enabled && cm.highlight);
    let mut body = String::new();
    for (i, line) in lines.iter().enumerate() {
        let n = start + i;
        let mut cls = String::from("bc-line");
        if marked.contains(&(i + 1)) {
            cls.push_str(" bc-line-mark");
        }
        if is_diff {
            let raw = code.lines().nth(i).unwrap_or("");
            if raw.starts_with('+') && !raw.starts_with("+++") {
                cls.push_str(" bc-line-add");
            } else if raw.starts_with('-') && !raw.starts_with("---") {
                cls.push_str(" bc-line-del");
            }
        }
        body.push_str(&format!(r#"<span class="{cls}" data-n="{n}">{line}</span>"#));
    }
    let title = info
        .get("title")
        .or(info.get("file"))
        .or(info.get("filename"))
        .map(|t| format!(r#"<div class="bc-code-title">{}</div>"#, escape(t)))
        .unwrap_or_default();
    let caption = info
        .get("caption")
        .map(|c| format!(r#"<figcaption class="bc-code-caption">{}</figcaption>"#, escape(c)))
        .unwrap_or_default();
    let id = info.id.as_ref().map(|i| format!(r#" id="{}""#, escape(i))).unwrap_or_default();
    let lang_label = if lang.is_empty() { "text".to_string() } else { safe_token(lang) };
    format!(
        r#"<figure class="bc-code{}{}{}"{id} data-lang="{lang_label}"{}{}>{title}<pre class="bc-pre"><code class="hl language-{lang_label}">{body}</code></pre>{caption}</figure>"#,
        if numbered { " bc-code-numbered" } else { "" },
        if wrap { " bc-code-wrap" } else { "" },
        if title.is_empty() { "" } else { " bc-code-has-title" },
        if runnable { r#" data-run="js""# } else { "" },
        if cm.copy_button { r#" data-copy="1""# } else { "" }
    ) + "\n"
}

fn terminal(code: &str, info: &Info, ctx: &Ctx) -> String {
    let cm = &ctx.opts.config.modules.code;
    let mut body = String::new();
    for raw in code.lines() {
        let prompt = ["$ ", "> ", "PS> ", "# ", "% "]
            .iter()
            .find(|p| raw.starts_with(**p))
            .copied()
            .or_else(|| {
                // `PS C:\path> command`
                raw.starts_with("PS ").then(|| raw.find("> ").map(|i| &raw[..i + 2])).flatten()
            });
        match prompt {
            Some(p) => {
                // Commands stay plain: the page's code theme is tuned for light backgrounds.
                body.push_str(&format!(
                    r#"<span class="bc-line bc-cmd"><span class="bc-prompt">{}</span>{}</span>"#,
                    escape(p),
                    escape(&raw[p.len()..])
                ));
            }
            None => body.push_str(&format!(r#"<span class="bc-line bc-out">{}</span>"#, escape(raw))),
        }
    }
    let title = info.get("title").unwrap_or("Terminal");
    format!(
        r#"<figure class="bc-code bc-terminal" data-lang="terminal"{}><div class="bc-terminal-bar"><span></span><span></span><span></span><div>{}</div></div><pre class="bc-pre"><code>{body}</code></pre></figure>"#,
        if cm.copy_button { r#" data-copy="1""# } else { "" },
        escape(title)
    ) + "\n"
}
