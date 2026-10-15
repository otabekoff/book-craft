//! ```` ```canvas ```` → SVG drawing from a tiny line-based language:
//!
//! ```text
//! size 640 300
//! grid 20
//! rect 20 20 160 80 fill=soft stroke=accent radius=10 label="Input"
//! circle 320 60 40 fill=accent label="Core" color=paper
//! arrow 180 60 280 60 stroke=ink
//! line 0 200 640 200 dash=6
//! text 320 260 "Force diagram" size=18 anchor=middle weight=bold
//! polygon "300,150 340,220 260,220" fill=orange
//! axes 40 260 200 160 x="t (s)" y="v (m/s)"
//! image 500 120 100 100 src="assets/photo.jpg"
//! ```
//! Colors are theme tokens (ink, muted, paper, accent, soft, line, red, orange,
//! yellow, green, teal, blue, purple, pink) or hex values.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::escape;

static CANVAS_ID: AtomicUsize = AtomicUsize::new(0);

struct Cmd {
    name: String,
    args: Vec<String>,
    opts: BTreeMap<String, String>,
    line: usize,
}

pub fn render(source: &str, caption: Option<&str>, asset_url: &dyn Fn(&str) -> String) -> Result<String, String> {
    let uid = CANVAS_ID.fetch_add(1, Ordering::Relaxed);
    let mut width = 640.0;
    let mut height = 360.0;
    let mut cmds = Vec::new();
    for (i, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let tokens = tokenize(line);
        let mut it = tokens.into_iter();
        let name = it.next().unwrap_or_default().to_ascii_lowercase();
        let mut args = Vec::new();
        let mut opts = BTreeMap::new();
        for t in it {
            match t.split_once('=') {
                Some((k, v)) if !k.contains('"') && !k.is_empty() => {
                    opts.insert(k.to_ascii_lowercase(), unquote(v));
                }
                _ => args.push(unquote(&t)),
            }
        }
        if name == "size" {
            width = num(&args, 0, i)?.clamp(40.0, 4000.0);
            height = num(&args, 1, i)?.clamp(40.0, 4000.0);
            continue;
        }
        cmds.push(Cmd { name, args, opts, line: i + 1 });
    }

    let mut defs = String::new();
    let mut markers: BTreeMap<String, String> = BTreeMap::new();
    let mut body = String::new();
    for c in &cmds {
        body.push_str(&draw(c, width, height, uid, &mut markers, &mut defs, asset_url)?);
    }
    // Drawings fill the text width; only small ones keep their size so they don't blow up.
    let limit = if width < 360.0 { format!(r#" style="max-width:{width}px""#) } else { String::new() };
    let mut html = format!(
        r#"<figure class="bc-canvas"><svg viewBox="0 0 {width} {height}" class="bc-canvas-svg" role="img" aria-label="{}"{limit}><defs>{defs}</defs>{body}</svg>"#,
        escape(caption.unwrap_or("Drawing"))
    );
    if let Some(c) = caption.filter(|c| !c.is_empty()) {
        html.push_str(&format!("<figcaption>{}</figcaption>", escape(c)));
    }
    html.push_str("</figure>");
    Ok(html)
}

fn draw(
    c: &Cmd,
    width: f64,
    height: f64,
    uid: usize,
    markers: &mut BTreeMap<String, String>,
    defs: &mut String,
    asset_url: &dyn Fn(&str) -> String,
) -> Result<String, String> {
    let l = c.line;
    let a = &c.args;
    let o = &c.opts;
    let mut marker_for = |color: &str| -> String {
        if let Some(id) = markers.get(color) {
            return id.clone();
        }
        let id = format!("bc-arrow-{uid}-{}", markers.len());
        defs.push_str(&format!(
            r#"<marker id="{id}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" fill="{color}"/></marker>"#
        ));
        markers.insert(color.to_string(), id.clone());
        id
    };
    let out = match c.name.as_str() {
        "grid" => {
            let step = a.first().and_then(|s| s.parse::<f64>().ok()).unwrap_or(20.0).max(4.0);
            let mut s = String::from(r#"<g class="bc-canvas-grid">"#);
            let mut x = 0.0;
            while x <= width {
                s.push_str(&format!(r#"<line x1="{x}" y1="0" x2="{x}" y2="{height}"/>"#));
                x += step;
            }
            let mut y = 0.0;
            while y <= height {
                s.push_str(&format!(r#"<line x1="0" y1="{y}" x2="{width}" y2="{y}"/>"#));
                y += step;
            }
            s.push_str("</g>");
            s
        }
        "rect" | "box" => {
            let (x, y, w, h) = (num(a, 0, l)?, num(a, 1, l)?, num(a, 2, l)?, num(a, 3, l)?);
            let r = opt_num(o, "radius").unwrap_or(0.0);
            let mut s = format!(
                r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" {}/>"#,
                paint(o, "soft", "ink")?
            );
            s.push_str(&label(o, x + w / 2.0, y + h / 2.0)?);
            s
        }
        "circle" => {
            let (cx, cy, r) = (num(a, 0, l)?, num(a, 1, l)?, num(a, 2, l)?);
            format!(r#"<circle cx="{cx}" cy="{cy}" r="{r}" {}/>{}"#, paint(o, "soft", "ink")?, label(o, cx, cy)?)
        }
        "ellipse" => {
            let (cx, cy, rx, ry) = (num(a, 0, l)?, num(a, 1, l)?, num(a, 2, l)?, num(a, 3, l)?);
            format!(
                r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" {}/>{}"#,
                paint(o, "soft", "ink")?,
                label(o, cx, cy)?
            )
        }
        "line" | "arrow" | "vector" => {
            let (x1, y1, x2, y2) = (num(a, 0, l)?, num(a, 1, l)?, num(a, 2, l)?, num(a, 3, l)?);
            let stroke = color(o.get("stroke").or(o.get("color")).map(String::as_str).unwrap_or("ink"))?;
            let w = opt_num(o, "width").unwrap_or(2.0);
            let mut m = String::new();
            if c.name != "line" || o.contains_key("end") {
                let id = marker_for(&stroke);
                m.push_str(&format!(r#" marker-end="url(#{id})""#));
                if o.get("both").is_some_and(|v| v != "false") || o.contains_key("start") {
                    m.push_str(&format!(r#" marker-start="url(#{id})""#));
                }
            }
            let mut s = format!(
                r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{w}" stroke-linecap="round"{}{m}/>"#,
                dash(o)
            );
            s.push_str(&label(o, (x1 + x2) / 2.0, (y1 + y2) / 2.0 - 10.0)?);
            s
        }
        "polyline" | "polygon" => {
            let pts = a.join(" ");
            let mut clean = Vec::new();
            for p in pts.split_whitespace() {
                let (x, y) = p.split_once(',').ok_or(format!("Line {l}: points look like 10,20 30,40"))?;
                let x: f64 = x.parse().map_err(|_| format!("Line {l}: bad point {p}"))?;
                let y: f64 = y.parse().map_err(|_| format!("Line {l}: bad point {p}"))?;
                clean.push(format!("{x},{y}"));
            }
            let fill_default = if c.name == "polygon" { "soft" } else { "none" };
            format!(r#"<{} points="{}" {}/>"#, c.name, clean.join(" "), paint(o, fill_default, "ink")?)
        }
        "path" => {
            let d = a.first().ok_or(format!("Line {l}: path needs a d string"))?;
            if !d.chars().all(|ch| ch.is_ascii_alphanumeric() || " ,.-+".contains(ch)) {
                return Err(format!("Line {l}: path data may only contain commands and numbers"));
            }
            format!(r#"<path d="{d}" {}/>"#, paint(o, "none", "ink")?)
        }
        "text" => {
            let (x, y) = (num(a, 0, l)?, num(a, 1, l)?);
            let text = a.get(2).cloned().unwrap_or_default();
            text_el(x, y, &text, o, "ink")?
        }
        "image" => {
            let (x, y, w, h) = (num(a, 0, l)?, num(a, 1, l)?, num(a, 2, l)?, num(a, 3, l)?);
            let src = o.get("src").ok_or(format!("Line {l}: image needs src=\"assets/…\""))?;
            format!(
                r#"<image href="{}" x="{x}" y="{y}" width="{w}" height="{h}" preserveAspectRatio="xMidYMid meet"/>"#,
                escape(&asset_url(src))
            )
        }
        "axes" => {
            let (x, y, w, h) = (num(a, 0, l)?, num(a, 1, l)?, num(a, 2, l)?, num(a, 3, l)?);
            let stroke = color(o.get("stroke").map(String::as_str).unwrap_or("ink"))?;
            let id = marker_for(&stroke);
            let mut s = format!(
                r#"<g stroke="{stroke}" stroke-width="1.6"><line x1="{x}" y1="{y}" x2="{}" y2="{y}" marker-end="url(#{id})"/><line x1="{x}" y1="{y}" x2="{x}" y2="{}" marker-end="url(#{id})"/></g>"#,
                x + w,
                y - h
            );
            if let Some(t) = o.get("x") {
                s.push_str(&text_el(x + w, y + 20.0, t, &BTreeMap::from([("anchor".into(), "end".into())]), "ink")?);
            }
            if let Some(t) = o.get("y") {
                s.push_str(&text_el(x + 8.0, y - h + 4.0, t, &BTreeMap::new(), "ink")?);
            }
            s
        }
        other => return Err(format!("Line {l}: unknown shape \"{other}\"")),
    };
    Ok(match opt_num(o, "rotate") {
        Some(deg) => {
            let cx = num(a, 0, l).unwrap_or(0.0);
            let cy = num(a, 1, l).unwrap_or(0.0);
            format!(r#"<g transform="rotate({deg} {cx} {cy})">{out}</g>"#)
        }
        None => out,
    })
}

fn paint(o: &BTreeMap<String, String>, fill: &str, stroke: &str) -> Result<String, String> {
    let fill = color(o.get("fill").map(String::as_str).unwrap_or(fill))?;
    let stroke = color(o.get("stroke").map(String::as_str).unwrap_or(stroke))?;
    let w = opt_num(o, "width").unwrap_or(1.6);
    let opacity = opt_num(o, "opacity").map(|v| format!(r#" opacity="{}""#, v.clamp(0.0, 1.0))).unwrap_or_default();
    Ok(format!(r#"fill="{fill}" stroke="{stroke}" stroke-width="{w}"{}{opacity}"#, dash(o)))
}

fn dash(o: &BTreeMap<String, String>) -> String {
    match o.get("dash") {
        Some(v) => {
            let n = v.parse::<f64>().unwrap_or(6.0).clamp(1.0, 50.0);
            format!(r#" stroke-dasharray="{n} {n}""#)
        }
        None => String::new(),
    }
}

fn label(o: &BTreeMap<String, String>, x: f64, y: f64) -> Result<String, String> {
    match o.get("label") {
        Some(t) => {
            let mut opts = BTreeMap::from([("anchor".to_string(), "middle".to_string())]);
            if let Some(c) = o.get("color") {
                opts.insert("fill".into(), c.clone());
            }
            if let Some(s) = o.get("size") {
                opts.insert("size".into(), s.clone());
            }
            text_el(x, y, t, &opts, "ink").map(|s| s.replace("<text ", r#"<text dominant-baseline="central" "#))
        }
        None => Ok(String::new()),
    }
}

fn text_el(x: f64, y: f64, text: &str, o: &BTreeMap<String, String>, fill: &str) -> Result<String, String> {
    let fill = color(o.get("fill").or(o.get("color")).map(String::as_str).unwrap_or(fill))?;
    let size = opt_num(o, "size").unwrap_or(15.0).clamp(6.0, 200.0);
    let anchor = match o.get("anchor").map(String::as_str) {
        Some("middle" | "center") => "middle",
        Some("end" | "right") => "end",
        _ => "start",
    };
    let weight = match o.get("weight").map(String::as_str) {
        Some("bold") => "700",
        Some("medium") => "500",
        _ => "400",
    };
    let style = if o.get("italic").is_some() { r#" font-style="italic""# } else { "" };
    Ok(format!(
        r#"<text x="{x}" y="{y}" fill="{fill}" font-size="{size}" font-weight="{weight}" text-anchor="{anchor}"{style}>{}</text>"#,
        escape(text)
    ))
}

pub fn color(v: &str) -> Result<String, String> {
    let token = v.trim().to_ascii_lowercase();
    let named = [
        "ink", "muted", "paper", "accent", "soft", "line", "red", "orange", "yellow", "green", "teal", "blue",
        "purple", "pink",
    ];
    if token == "none" || token == "transparent" {
        return Ok("none".into());
    }
    if named.contains(&token.as_str()) {
        return Ok(format!("var(--bc-c-{token})"));
    }
    let hex = token.strip_prefix('#').unwrap_or("");
    if !hex.is_empty() && matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(token);
    }
    Err(format!("Unknown color \"{v}\" (use a name like accent or a hex value like #2f5a8a)"))
}

fn num(args: &[String], i: usize, line: usize) -> Result<f64, String> {
    args.get(i)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .ok_or(format!("Line {line}: expected a number in position {}", i + 1))
}

fn opt_num(o: &BTreeMap<String, String>, key: &str) -> Option<f64> {
    o.get(key).and_then(|v| v.parse::<f64>().ok()).filter(|v| v.is_finite())
}

fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for ch in line.chars() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                cur.push(ch);
            }
            c if c.is_whitespace() && !in_quote => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn unquote(s: &str) -> String {
    s.trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_shapes() {
        let src = "size 300 200\nrect 10 10 100 50 fill=accent label=\"Hi <b>\"\narrow 0 0 50 50\ntext 5 5 \"x\" size=20";
        let svg = render(src, Some("Test"), &|s| format!("/{s}")).unwrap();
        assert!(svg.contains(r#"viewBox="0 0 300 200""#));
        assert!(svg.contains("var(--bc-c-accent)"));
        assert!(svg.contains("Hi &lt;b&gt;"));
        assert!(svg.contains("marker-end"));
    }

    #[test]
    fn rejects_injection() {
        assert!(render("rect 0 0 1 1 fill=\"red\" onload=alert(1)", None, &|s| s.into()).unwrap().contains("rect"));
        assert!(render("rect 0 0 1 1 fill=\"x\\\" onload=\"", None, &|s| s.into()).is_err());
        assert!(render("path \"M0 0\\\" onload=\\\"x\"", None, &|s| s.into()).is_err());
    }
}
