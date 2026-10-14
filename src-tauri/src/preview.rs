//! `bookasset://` serves the live preview: the book's stylesheet and scripts,
//! the math/diagram libraries and files from the open project.
//! On Windows the preview loads it as `http://bookasset.localhost/…`.

use std::borrow::Cow;

use book_core::project::resolve_inside;
use book_core::theme;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Manager};

use crate::state::AppState;

pub const PROJECT_BASE: &str = "http://bookasset.localhost/project/";

fn mime(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "html" => "text/html; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

fn reply(status: StatusCode, ct: &str, body: Vec<u8>) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, ct)
        // The preview frame is sandboxed (opaque origin); fonts need CORS.
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Cow::Owned(body))
        .unwrap()
}

fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn serve(app: &AppHandle, req: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let state = app.state::<AppState>();
    let path = decode(req.uri().path());
    let path = path.trim_start_matches('/');
    let not_found = || reply(StatusCode::NOT_FOUND, "text/plain", b"not found".to_vec());

    match path {
        "_runtime/book.css" => {
            let guard = state.project.lock().unwrap();
            let css = match guard.as_ref() {
                Some(p) => theme::book_css(&p.config, &p.root, PROJECT_BASE),
                None => theme::BLOCKS_CSS.to_string(),
            };
            reply(StatusCode::OK, "text/css; charset=utf-8", css.into_bytes())
        }
        "_runtime/blocks.js" => reply(StatusCode::OK, "text/javascript; charset=utf-8", theme::BLOCKS_JS.as_bytes().to_vec()),
        p if p.starts_with("_runtime/vendor/") => {
            let Some(vendor) = state.resources.vendor.as_ref() else { return not_found() };
            match resolve_inside(vendor, &p["_runtime/vendor/".len()..]).ok().and_then(|f| std::fs::read(f).ok()) {
                Some(data) => reply(StatusCode::OK, mime(p), data),
                None => not_found(),
            }
        }
        p if p.starts_with("project/") => {
            let root = state.project.lock().unwrap().as_ref().map(|p| p.root.clone());
            let Some(root) = root else { return not_found() };
            let rel = &p["project/".len()..];
            // Only book content is served: assets and chapters, never book.json or dist.
            if !(rel.starts_with("assets/") || rel.starts_with("chapters/")) {
                return not_found();
            }
            match resolve_inside(&root, rel).ok().and_then(|f| std::fs::read(f).ok()) {
                Some(data) => reply(StatusCode::OK, mime(rel), data),
                None => not_found(),
            }
        }
        _ => not_found(),
    }
}
