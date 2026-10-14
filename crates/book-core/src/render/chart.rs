//! ```` ```chart {type=bar title="Readers" y="Thousands"} ```` → static SVG.
//!
//! Data is CSV (first row: header, first column: labels) or JSON:
//! `{"labels":["Q1","Q2"],"series":[{"name":"2024","values":[3,5]}]}`.
//! Colors come from CSS variables so charts follow the reader theme.

use serde::Deserialize;

use super::attrs::Info;
use super::escape;

#[derive(Debug, Default, Deserialize)]
struct Data {
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    series: Vec<Series>,
}

#[derive(Debug, Default, Deserialize)]
struct Series {
    #[serde(default)]
    name: String,
    #[serde(default)]
    values: Vec<f64>,
}

const W: f64 = 640.0;

pub fn render(source: &str, info: &Info) -> Result<String, String> {
    let data = parse_data(source)?;
    if data.series.is_empty() || data.series.iter().all(|s| s.values.is_empty()) {
        return Err("The chart has no numbers.".into());
    }
    let kind = info.get_or("type", "bar").to_ascii_lowercase();
    let h = info.number("height").unwrap_or(320.0).clamp(160.0, 900.0);
    let title = info.get("title").unwrap_or("");
    let body = match kind.as_str() {
        "pie" | "donut" => pie(&data, h, kind == "donut"),
        "line" | "area" => cartesian(&data, info, h, if kind == "area" { Mode::Area } else { Mode::Line }),
        "hbar" => hbar(&data, info, h),
        "stacked" => cartesian(&data, info, h, Mode::Stacked),
        _ => cartesian(&data, info, h, Mode::Bar),
    };
    let legend = if data.series.len() > 1 || matches!(kind.as_str(), "pie" | "donut") {
        legend(&data, matches!(kind.as_str(), "pie" | "donut"))
    } else {
        String::new()
    };
    let label = if title.is_empty() { "Chart".to_string() } else { title.to_string() };
    let mut html = format!(
        r#"<figure class="bc-chart"><svg class="bc-chart-svg" viewBox="0 0 {W} {h}" role="img" aria-label="{}">{body}</svg>{legend}"#,
        escape(&label)
    );
    let caption = info.get("caption").unwrap_or(title);
    if !caption.is_empty() {
        html.push_str(&format!("<figcaption>{}</figcaption>", escape(caption)));
    }
    html.push_str("</figure>");
    Ok(html)
}

fn parse_data(source: &str) -> Result<Data, String> {
    let trimmed = source.trim();
    if trimmed.starts_with('{') {
        return serde_json::from_str(trimmed).map_err(|e| format!("Chart JSON: {e}"));
    }
    let rows: Vec<Vec<String>> = trimmed
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|l| l.split([',', '\t', ';']).map(|c| c.trim().to_string()).collect())
        .collect();
    if rows.len() < 2 {
        return Err("A chart needs a header row and at least one data row.".into());
    }
    let header = &rows[0];
    let mut data = Data::default();
    for name in header.iter().skip(1) {
        data.series.push(Series { name: name.clone(), values: Vec::new() });
    }
    if data.series.is_empty() {
        data.series.push(Series { name: header[0].clone(), values: Vec::new() });
    }
    for row in &rows[1..] {
        data.labels.push(row.first().cloned().unwrap_or_default());
        for (i, s) in data.series.iter_mut().enumerate() {
            let v = row.get(i + 1).and_then(|c| c.replace(' ', "").parse::<f64>().ok()).unwrap_or(0.0);
            s.values.push(v);
        }
    }
    Ok(data)
}

fn color(i: usize) -> String {
    format!("var(--bc-series-{})", i % 8 + 1)
}

fn nice_max(v: f64) -> f64 {
    if v <= 0.0 {
        return 1.0;
    }
    let mag = 10f64.powf(v.log10().floor());
    let n = v / mag;
    let step = if n <= 1.0 { 1.0 } else if n <= 2.0 { 2.0 } else if n <= 5.0 { 5.0 } else { 10.0 };
    step * mag
}

fn fmt_num(v: f64) -> String {
    if v.abs() >= 1000.0 && (v / 1000.0).fract() == 0.0 {
        format!("{}k", v / 1000.0)
    } else if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Bar,
    Stacked,
    Line,
    Area,
}

fn cartesian(data: &Data, info: &Info, h: f64, mode: Mode) -> String {
    let (left, right, top, bottom) = (56.0, 16.0, 16.0, 40.0);
    let pw = W - left - right;
    let ph = h - top - bottom;
    let n = data.labels.len().max(data.series.iter().map(|s| s.values.len()).max().unwrap_or(0)).max(1);
    let value = |s: &Series, i: usize| s.values.get(i).copied().unwrap_or(0.0);
    let raw_max = if mode == Mode::Stacked {
        (0..n).map(|i| data.series.iter().map(|s| value(s, i).max(0.0)).sum::<f64>()).fold(0.0, f64::max)
    } else {
        data.series.iter().flat_map(|s| s.values.iter().copied()).fold(0.0, f64::max)
    };
    let raw_min = data.series.iter().flat_map(|s| s.values.iter().copied()).fold(0.0, f64::min);
    let max = info.number("max").unwrap_or_else(|| nice_max(raw_max));
    let min = info.number("min").unwrap_or(if raw_min < 0.0 { -nice_max(-raw_min) } else { 0.0 });
    let span = (max - min).max(f64::EPSILON);
    let y = |v: f64| top + ph - (v - min) / span * ph;

    let mut svg = String::new();
    // Grid and y labels.
    for t in 0..=4 {
        let v = min + span * t as f64 / 4.0;
        let yy = y(v);
        svg.push_str(&format!(
            r#"<line class="bc-chart-grid" x1="{left}" x2="{}" y1="{yy:.1}" y2="{yy:.1}"/><text class="bc-chart-tick" x="{}" y="{:.1}" text-anchor="end">{}</text>"#,
            W - right,
            left - 8.0,
            yy + 4.0,
            fmt_num(v)
        ));
    }
    if let Some(label) = info.get("y") {
        svg.push_str(&format!(
            r#"<text class="bc-chart-axis" transform="translate(14 {:.1}) rotate(-90)" text-anchor="middle">{}</text>"#,
            top + ph / 2.0,
            escape(label)
        ));
    }
    let band = pw / n as f64;
    for (i, label) in data.labels.iter().enumerate().take(n) {
        let x = left + band * (i as f64 + 0.5);
        svg.push_str(&format!(
            r#"<text class="bc-chart-tick" x="{x:.1}" y="{:.1}" text-anchor="middle">{}</text>"#,
            h - bottom + 18.0,
            escape(label)
        ));
    }
    if let Some(label) = info.get("x") {
        svg.push_str(&format!(
            r#"<text class="bc-chart-axis" x="{:.1}" y="{:.1}" text-anchor="middle">{}</text>"#,
            left + pw / 2.0,
            h - 4.0,
            escape(label)
        ));
    }

    let zero = y(0f64.clamp(min, max));
    match mode {
        Mode::Bar => {
            let groups = data.series.len().max(1) as f64;
            let bw = (band * 0.72) / groups;
            for (si, s) in data.series.iter().enumerate() {
                for i in 0..n {
                    let v = value(s, i);
                    let x = left + band * i as f64 + band * 0.14 + bw * si as f64;
                    let (y0, y1) = if v >= 0.0 { (y(v), zero) } else { (zero, y(v)) };
                    svg.push_str(&format!(
                        r#"<rect x="{x:.1}" y="{y0:.1}" width="{:.1}" height="{:.1}" rx="2" fill="{}"><title>{}: {}</title></rect>"#,
                        (bw - 2.0).max(1.0),
                        (y1 - y0).max(0.5),
                        color(si),
                        escape(&s.name),
                        fmt_num(v)
                    ));
                }
            }
        }
        Mode::Stacked => {
            let bw = band * 0.6;
            for i in 0..n {
                let mut acc = 0.0;
                for (si, s) in data.series.iter().enumerate() {
                    let v = value(s, i).max(0.0);
                    let y0 = y(acc + v);
                    let y1 = y(acc);
                    acc += v;
                    svg.push_str(&format!(
                        r#"<rect x="{:.1}" y="{y0:.1}" width="{bw:.1}" height="{:.1}" fill="{}"><title>{}: {}</title></rect>"#,
                        left + band * i as f64 + band * 0.2,
                        (y1 - y0).max(0.0),
                        color(si),
                        escape(&s.name),
                        fmt_num(v)
                    ));
                }
            }
        }
        Mode::Line | Mode::Area => {
            for (si, s) in data.series.iter().enumerate() {
                let pts: Vec<(f64, f64)> = (0..n)
                    .map(|i| (left + band * (i as f64 + 0.5), y(value(s, i))))
                    .collect();
                let path: Vec<String> = pts.iter().map(|(x, y)| format!("{x:.1},{y:.1}")).collect();
                if mode == Mode::Area {
                    if let (Some(first), Some(last)) = (pts.first(), pts.last()) {
                        svg.push_str(&format!(
                            r#"<polygon points="{:.1},{zero:.1} {} {:.1},{zero:.1}" fill="{}" opacity="0.18"/>"#,
                            first.0,
                            path.join(" "),
                            last.0,
                            color(si)
                        ));
                    }
                }
                svg.push_str(&format!(
                    r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>"#,
                    path.join(" "),
                    color(si)
                ));
                for (i, (x, yy)) in pts.iter().enumerate() {
                    svg.push_str(&format!(
                        r#"<circle cx="{x:.1}" cy="{yy:.1}" r="3.5" fill="{}"><title>{}: {}</title></circle>"#,
                        color(si),
                        escape(&s.name),
                        fmt_num(value(s, i))
                    ));
                }
            }
        }
    }
    svg.push_str(&format!(
        r#"<line class="bc-chart-axisline" x1="{left}" x2="{}" y1="{zero:.1}" y2="{zero:.1}"/>"#,
        W - right
    ));
    svg
}

fn hbar(data: &Data, info: &Info, h: f64) -> String {
    let (left, right, top, bottom) = (120.0, 48.0, 12.0, 12.0);
    let s = &data.series[0];
    let n = data.labels.len().max(s.values.len()).max(1);
    let max = info.number("max").unwrap_or_else(|| nice_max(s.values.iter().copied().fold(0.0, f64::max)));
    let band = (h - top - bottom) / n as f64;
    let pw = W - left - right;
    let mut svg = String::new();
    for i in 0..n {
        let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
        let y = top + band * i as f64;
        let w = v / max.max(f64::EPSILON) * pw;
        let label = data.labels.get(i).map(String::as_str).unwrap_or("");
        svg.push_str(&format!(
            r#"<text class="bc-chart-tick" x="{:.1}" y="{:.1}" text-anchor="end">{}</text><rect x="{left}" y="{:.1}" width="{w:.1}" height="{:.1}" rx="2" fill="{}"/><text class="bc-chart-tick" x="{:.1}" y="{:.1}">{}</text>"#,
            left - 8.0,
            y + band / 2.0 + 4.0,
            escape(label),
            y + band * 0.18,
            band * 0.64,
            color(0),
            left + w + 6.0,
            y + band / 2.0 + 4.0,
            fmt_num(v)
        ));
    }
    svg
}

fn pie(data: &Data, h: f64, donut: bool) -> String {
    let s = &data.series[0];
    let total: f64 = s.values.iter().map(|v| v.max(0.0)).sum();
    if total <= 0.0 {
        return String::new();
    }
    let (cx, cy, r) = (W / 2.0, h / 2.0, h / 2.0 - 12.0);
    let mut angle = -std::f64::consts::FRAC_PI_2;
    let mut svg = String::new();
    for (i, v) in s.values.iter().enumerate() {
        let v = v.max(0.0);
        if v == 0.0 {
            continue;
        }
        let sweep = v / total * std::f64::consts::TAU;
        let label = data.labels.get(i).map(String::as_str).unwrap_or("");
        let pct = v / total * 100.0;
        if sweep >= std::f64::consts::TAU - 1e-9 {
            svg.push_str(&format!(r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{}"/>"#, color(i)));
        } else {
            let (x0, y0) = (cx + r * angle.cos(), cy + r * angle.sin());
            let end = angle + sweep;
            let (x1, y1) = (cx + r * end.cos(), cy + r * end.sin());
            let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
            svg.push_str(&format!(
                r#"<path d="M{cx},{cy} L{x0:.2},{y0:.2} A{r},{r} 0 {large} 1 {x1:.2},{y1:.2} Z" fill="{}" class="bc-chart-slice"><title>{}: {} ({pct:.0}%)</title></path>"#,
                color(i),
                escape(label),
                fmt_num(v)
            ));
        }
        angle += sweep;
    }
    if donut {
        svg.push_str(&format!(r#"<circle cx="{cx}" cy="{cy}" r="{:.1}" class="bc-chart-hole"/>"#, r * 0.55));
    }
    svg
}

fn legend(data: &Data, by_label: bool) -> String {
    let mut out = String::from(r#"<ul class="bc-chart-legend">"#);
    if by_label {
        for (i, l) in data.labels.iter().enumerate() {
            out.push_str(&format!(r#"<li><span style="background:{}"></span>{}</li>"#, color(i), escape(l)));
        }
    } else {
        for (i, s) in data.series.iter().enumerate() {
            out.push_str(&format!(r#"<li><span style="background:{}"></span>{}</li>"#, color(i), escape(&s.name)));
        }
    }
    out.push_str("</ul>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::attrs::parse;

    #[test]
    fn csv_bar() {
        let svg = render("Quarter, 2023, 2024\nQ1, 10, 12\nQ2, 14, 18\n", &parse("chart {title=Sales}")).unwrap();
        assert!(svg.contains("<svg"));
        assert_eq!(svg.matches("<rect").count(), 4);
        assert!(svg.contains("bc-chart-legend"));
    }

    #[test]
    fn json_pie() {
        let svg = render(r#"{"labels":["a","b"],"series":[{"values":[1,3]}]}"#, &parse("chart {type=pie}")).unwrap();
        assert_eq!(svg.matches("<path").count(), 2);
    }
}
