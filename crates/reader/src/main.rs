//! Book reader runtime.
//!
//! This single binary is the template for every generated book. It finds its
//! `book.pak` (next to the exe, or appended to it), takes its identity from the
//! book's manifest and serves the book to the webview over `book://`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use pak_format::Slice;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Url, WebviewUrl, WebviewWindowBuilder};
use zip::ZipArchive;

type Archive = ZipArchive<Slice>;

const ENTRY: &str = "http://book.localhost/_reader/index.html";

const CSP: &str = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; \
img-src 'self' data: blob: https:; media-src 'self' blob: https:; font-src 'self' data:; \
connect-src 'self'; frame-src 'self' data: blob: about:; object-src 'none'; base-uri 'self'";

fn open_pak() -> Result<(Archive, String), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // `--book <path>` is for development and for the studio's preview.
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--book" {
            if let Some(p) = args.next() {
                return open_file(PathBuf::from(p));
            }
        }
    }
    if let Some(dir) = exe.parent() {
        let beside = dir.join(pak_format::PAK_FILE);
        if beside.is_file() {
            return open_file(beside);
        }
    }
    let mut f = File::open(&exe).map_err(|e| e.to_string())?;
    match pak_format::find(&mut f).map_err(|e| e.to_string())? {
        Some((offset, len)) => {
            let slice = Slice::new(f, offset, len).map_err(|e| e.to_string())?;
            let zip = ZipArchive::new(slice).map_err(|e| format!("The book inside this app is damaged: {e}"))?;
            Ok((zip, exe.display().to_string()))
        }
        None => Err(format!(
            "No book was found.\n\nThis is the Book Craft reader template. Put a {} file next to it, or start it with --book <path>.",
            pak_format::PAK_FILE
        )),
    }
}

fn open_file(path: PathBuf) -> Result<(Archive, String), String> {
    let f = File::open(&path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
    let len = f.metadata().map_err(|e| e.to_string())?.len();
    let slice = Slice::new(f, 0, len).map_err(|e| e.to_string())?;
    let zip = ZipArchive::new(slice).map_err(|e| format!("{} is not a valid book: {e}", path.display()))?;
    Ok((zip, path.display().to_string()))
}

fn read_entry(zip: &mut Archive, name: &str) -> Option<Vec<u8>> {
    let mut f = zip.by_name(name).ok()?;
    let mut buf = Vec::with_capacity(f.size() as usize);
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

fn mime(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" | "md" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "mp3" => "audio/mpeg",
        "m4a" | "aac" => "audio/mp4",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "pdf" => "application/pdf",
        "glb" => "model/gltf-binary",
        "gltf" => "model/gltf+json",
        _ => "application/octet-stream",
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn respond(status: StatusCode, content_type: &str, body: Vec<u8>) -> Response<Cow<'static, [u8]>> {
    let mut b = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::CACHE_CONTROL, "no-cache")
        .header("X-Content-Type-Options", "nosniff");
    if content_type.starts_with("text/html") {
        b = b.header("Content-Security-Policy", CSP);
    }
    b.body(Cow::Owned(body)).unwrap()
}

/// `bytes=a-b` → inclusive range within `len`.
fn parse_range(value: &str, len: usize) -> Option<(usize, usize)> {
    let spec = value.strip_prefix("bytes=")?.split(',').next()?.trim();
    let (a, b) = spec.split_once('-')?;
    if len == 0 {
        return None;
    }
    let (start, end) = if a.is_empty() {
        let n: usize = b.parse().ok()?;
        (len.saturating_sub(n), len - 1)
    } else {
        let start: usize = a.parse().ok()?;
        let end = if b.is_empty() { len - 1 } else { b.parse::<usize>().ok()?.min(len - 1) };
        (start, end)
    };
    (start <= end && start < len).then_some((start, end))
}

fn serve<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    book: &Mutex<Archive>,
    req: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let path = percent_decode(req.uri().path());
    let path = path.trim_start_matches('/');
    if path.split('/').any(|seg| seg == "..") {
        return respond(StatusCode::BAD_REQUEST, "text/plain", b"bad path".to_vec());
    }

    // The reader UI is compiled into the exe.
    if let Some(rest) = path.strip_prefix("_reader/") {
        let rest = if rest.is_empty() { "index.html" } else { rest };
        return match app.asset_resolver().get(rest.to_string()) {
            Some(asset) => respond(StatusCode::OK, mime(rest), asset.bytes),
            None => respond(StatusCode::NOT_FOUND, "text/plain", b"not found".to_vec()),
        };
    }

    let data = {
        let mut zip = book.lock().unwrap();
        read_entry(&mut zip, path)
    };
    let Some(data) = data else {
        return respond(StatusCode::NOT_FOUND, "text/plain", format!("{path} is not in this book").into_bytes());
    };
    let ct = mime(path);
    if let Some(range) = req.headers().get(header::RANGE).and_then(|v| v.to_str().ok()) {
        if let Some((start, end)) = parse_range(range, data.len()) {
            let total = data.len();
            let body = data[start..=end].to_vec();
            return Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, ct)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{total}"))
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Cow::Owned(body))
                .unwrap();
        }
    }
    let mut resp = respond(StatusCode::OK, ct, data);
    resp.headers_mut().insert(header::ACCEPT_RANGES, header::HeaderValue::from_static("bytes"));
    resp
}

#[cfg(windows)]
fn show_error(message: &str) {
    use std::ffi::c_void;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(hwnd: *mut c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }
    let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
    let (text, caption) = (wide(message), wide("Book"));
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
    }
}

#[cfg(not(windows))]
fn show_error(message: &str) {
    eprintln!("{message}");
}

fn main() {
    let (mut archive, _source) = match open_pak() {
        Ok(v) => v,
        Err(e) => {
            show_error(&e);
            std::process::exit(1);
        }
    };
    let manifest: serde_json::Value = match read_entry(&mut archive, "book.json").map(|b| serde_json::from_slice(&b)) {
        Some(Ok(v)) => v,
        _ => {
            show_error("This book is missing its table of contents (book.json).");
            std::process::exit(1);
        }
    };
    let str_at = |ptr: &str, default: &str| -> String {
        manifest.pointer(ptr).and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()).unwrap_or(default).to_string()
    };
    let num_at = |ptr: &str, default: f64| manifest.pointer(ptr).and_then(|v| v.as_f64()).unwrap_or(default);
    let title = str_at("/meta/title", "Book");
    let identifier = str_at("/meta/packageId", "com.bookcraft.book");
    let width = num_at("/reader/windowWidth", 1200.0).clamp(320.0, 7680.0);
    let height = num_at("/reader/windowHeight", 820.0).clamp(240.0, 4320.0);
    let min_w = num_at("/reader/minWidth", 480.0).clamp(240.0, width);
    let min_h = num_at("/reader/minHeight", 400.0).clamp(200.0, height);
    let external = manifest.pointer("/reader/openLinksExternally").and_then(|v| v.as_bool()).unwrap_or(true);
    let icon = read_entry(&mut archive, "_app/icon.png").and_then(|b| tauri::image::Image::from_bytes(&b).ok());

    let mut context = tauri::generate_context!();
    // Each book is its own app: separate settings, storage and WebView2 profile.
    context.config_mut().identifier = identifier;
    context.config_mut().product_name = Some(title.clone());
    if let Some(icon) = icon.clone() {
        context.set_default_window_icon(Some(icon.to_owned()));
    }

    let book = Arc::new(Mutex::new(archive));
    let result = tauri::Builder::default()
        .register_uri_scheme_protocol("book", move |ctx, req| serve(ctx.app_handle(), &book, req))
        .setup(move |app| {
            let url: Url = ENTRY.parse()?;
            let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::CustomProtocol(url))
                .title(&title)
                .inner_size(width, height)
                .min_inner_size(min_w, min_h)
                .center()
                .visible(true)
                .on_navigation(move |url| {
                    if url.host_str() == Some("book.localhost") || url.scheme() == "about" || url.scheme() == "data" {
                        return true;
                    }
                    if external && matches!(url.scheme(), "http" | "https" | "mailto") {
                        let _ = tauri_plugin_opener::open_url(url.as_str(), None::<&str>);
                    }
                    false
                });
            if let Some(icon) = icon.clone() {
                builder = builder.icon(icon)?;
            }
            builder.build()?;
            Ok(())
        })
        .run(context);
    if let Err(e) = result {
        show_error(&format!("The book could not start: {e}\n\nIt needs Microsoft Edge WebView2. Install it from microsoft.com/edge/webview2 and try again."));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        assert_eq!(parse_range("bytes=0-9", 100), Some((0, 9)));
        assert_eq!(parse_range("bytes=90-", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=200-300", 100), None);
    }

    #[test]
    fn decode() {
        assert_eq!(percent_decode("/assets/my%20cat.png"), "/assets/my cat.png");
        assert_eq!(percent_decode("/a%2"), "/a%2");
    }
}
