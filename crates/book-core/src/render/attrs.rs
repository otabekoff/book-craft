//! Parses info strings such as `callout {type=warning title="Mind the gap" #id .wide open}`.

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Info {
    /// The first word (block name or language).
    pub name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub pairs: Vec<(String, String)>,
    /// Words after the name that are not inside `{}` (e.g. a title).
    pub rest: String,
}

impl Info {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs.iter().rev().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    }

    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).filter(|v| !v.is_empty()).unwrap_or(default)
    }

    pub fn flag(&self, key: &str) -> bool {
        matches!(self.get(key).map(|v| v.to_ascii_lowercase()).as_deref(), Some("" | "true" | "yes" | "1" | "on"))
    }

    pub fn flag_or(&self, key: &str, default: bool) -> bool {
        match self.get(key).map(|v| v.to_ascii_lowercase()) {
            None => default,
            Some(v) => matches!(v.as_str(), "" | "true" | "yes" | "1" | "on"),
        }
    }

    pub fn number(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.trim().trim_end_matches("px").parse().ok())
    }
}

pub fn parse(info: &str) -> Info {
    let info = info.trim();
    let (head, attrs) = match (info.find('{'), info.rfind('}')) {
        (Some(open), Some(close)) if close > open => (&info[..open], Some(&info[open + 1..close])),
        _ => (info, None),
    };
    let mut out = Info::default();
    let mut words = head.split_whitespace();
    out.name = words.next().unwrap_or("").to_ascii_lowercase();
    out.rest = words.collect::<Vec<_>>().join(" ");
    if let Some(attrs) = attrs {
        for token in tokenize(attrs) {
            if let Some(id) = token.strip_prefix('#') {
                out.id = Some(id.to_string());
            } else if let Some(class) = token.strip_prefix('.') {
                out.classes.push(class.to_string());
            } else if let Some((k, v)) = token.split_once('=') {
                out.pairs.push((k.trim().to_string(), unquote(v.trim())));
            } else if !token.is_empty() {
                out.pairs.push((token, String::new()));
            }
        }
    }
    out
}

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) if c == '\\' => {
                if let Some(&n) = chars.peek() {
                    if n == q || n == '\\' {
                        cur.push('\\');
                        cur.push(n);
                        chars.next();
                        continue;
                    }
                }
                cur.push(c);
            }
            Some(q) if c == q => {
                cur.push(c);
                quote = None;
            }
            Some(_) => cur.push(c),
            None if c == '"' || c == '\'' => {
                cur.push(c);
                quote = Some(c);
            }
            None if c.is_whitespace() || c == ',' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            None => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn unquote(v: &str) -> String {
    let bytes = v.as_bytes();
    if bytes.len() >= 2 && (bytes[0] == b'"' || bytes[0] == b'\'') && bytes[bytes.len() - 1] == bytes[0] {
        let q = bytes[0] as char;
        v[1..v.len() - 1].replace(&format!("\\{q}"), &q.to_string()).replace("\\\\", "\\")
    } else {
        v.to_string()
    }
}

/// Parses `2,4-6` into a sorted list of line numbers.
pub fn line_ranges(spec: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for part in spec.split([',', ' ']).filter(|p| !p.is_empty()) {
        if let Some((a, b)) = part.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.trim().parse::<usize>(), b.trim().parse::<usize>()) {
                if a <= b && b - a < 10_000 {
                    out.extend(a..=b);
                }
            }
        } else if let Ok(n) = part.trim().parse() {
            out.push(n);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_everything() {
        let i = parse(r#"callout My title {type=warning title="Mind \"the\" gap" #x .wide open}"#);
        assert_eq!(i.name, "callout");
        assert_eq!(i.rest, "My title");
        assert_eq!(i.get("type"), Some("warning"));
        assert_eq!(i.get("title"), Some(r#"Mind "the" gap"#));
        assert_eq!(i.id.as_deref(), Some("x"));
        assert_eq!(i.classes, vec!["wide"]);
        assert!(i.flag("open"));
        assert!(!i.flag("missing"));
    }

    #[test]
    fn ranges() {
        assert_eq!(line_ranges("4-6, 2,2"), vec![2, 4, 5, 6]);
    }
}
