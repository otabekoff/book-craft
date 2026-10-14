//! Commands the studio UI calls. Every path the UI sends is resolved inside the
//! open project; nothing outside it is read or written, except files the user
//! picked in a dialog (imports, certificate) and the build output folder.

use std::path::PathBuf;

use book_core::config::BookConfig;
use book_core::package::{self, BuildReport, BuildRequest, Target};
use book_core::project::{AssetInfo, ChapterInfo, ConfigFormat};
use book_core::render::{Heading, RenderOptions, render_markdown};
use book_core::signing::{CertificateInfo, TestCertificate};
use book_core::{Project, icons, pak, signing, theme};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::{AppState, StudioSettings};

type R<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn with_project<T>(state: &AppState, f: impl FnOnce(&mut Project) -> anyhow::Result<T>) -> R<T> {
    let mut guard = state.project.lock().unwrap();
    let project = guard.as_mut().ok_or("No book is open")?;
    f(project).map_err(|e| format!("{e:#}"))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    reader_template: Option<String>,
    vendor: Option<String>,
    tools: Vec<book_core::tools::ToolStatus>,
    code_themes: Vec<String>,
    fonts: Vec<(String, String)>,
    settings: StudioSettings,
}

#[tauri::command]
pub fn app_info(state: State<AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        reader_template: state.resources.reader.as_ref().map(|p| p.display().to_string()),
        vendor: state.resources.vendor.as_ref().map(|p| p.display().to_string()),
        tools: state.tools().status(),
        code_themes: book_core::render::highlight::theme_names(),
        fonts: theme::FONTS.iter().map(|(k, l, _)| (k.to_string(), l.to_string())).collect(),
        settings: state.settings.lock().unwrap().clone(),
    }
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: StudioSettings) -> AppInfo {
    *state.settings.lock().unwrap() = settings;
    state.save_settings();
    app_info(state)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectData {
    root: String,
    config: BookConfig,
    chapters: Vec<ChapterInfo>,
    chapter_class: String,
    problems: Vec<String>,
    format: ConfigFormat,
}

fn project_data(p: &Project) -> ProjectData {
    ProjectData {
        format: p.format,
        root: p.root.display().to_string(),
        config: p.config.clone(),
        chapters: p.chapters(),
        chapter_class: theme::chapter_classes(&p.config),
        problems: p.config.validate(),
    }
}

#[tauri::command]
pub fn project_create(state: State<AppState>, dir: String, title: String, author: String) -> R<ProjectData> {
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).map_err(err)?;
    let p = Project::create(&dir, &title, &author).map_err(|e| format!("{e:#}"))?;
    state.remember(&p.root);
    let data = project_data(&p);
    *state.project.lock().unwrap() = Some(p);
    Ok(data)
}

#[tauri::command]
pub fn project_open(state: State<AppState>, dir: String) -> R<ProjectData> {
    let p = Project::open(&dir).map_err(|e| format!("{e:#}"))?;
    state.remember(&p.root);
    let data = project_data(&p);
    *state.project.lock().unwrap() = Some(p);
    Ok(data)
}

#[tauri::command]
pub fn project_close(state: State<AppState>) {
    *state.project.lock().unwrap() = None;
}

#[tauri::command]
pub fn project_save_config(state: State<AppState>, config: BookConfig) -> R<ProjectData> {
    with_project(&state, |p| {
        p.config = config;
        p.save()?;
        Ok(project_data(p))
    })
}

#[tauri::command]
pub fn project_validate(state: State<AppState>) -> R<Vec<String>> {
    with_project(&state, |p| Ok(p.config.validate()))
}

#[tauri::command]
pub fn chapter_read(state: State<AppState>, file: String) -> R<String> {
    with_project(&state, |p| p.read_chapter(&file))
}

#[tauri::command]
pub fn chapter_write(state: State<AppState>, file: String, content: String) -> R<Vec<ChapterInfo>> {
    with_project(&state, |p| {
        if !p.config.chapters.iter().any(|c| c.file == file) {
            anyhow::bail!("{file} is not a chapter of this book");
        }
        p.write_chapter(&file, &content)?;
        Ok(p.chapters())
    })
}

#[tauri::command]
pub fn chapter_add(state: State<AppState>, title: String, after: Option<String>) -> R<(String, ProjectData)> {
    with_project(&state, |p| {
        let file = p.add_chapter(&title, after.as_deref())?;
        Ok((file, project_data(p)))
    })
}

#[tauri::command]
pub fn chapter_remove(state: State<AppState>, file: String) -> R<ProjectData> {
    with_project(&state, |p| {
        p.remove_chapter(&file)?;
        Ok(project_data(p))
    })
}

#[tauri::command]
pub fn assets_list(state: State<AppState>) -> R<Vec<AssetInfo>> {
    with_project(&state, |p| p.assets())
}

#[tauri::command]
pub fn assets_import(state: State<AppState>, paths: Vec<String>, subdir: String) -> R<Vec<String>> {
    with_project(&state, |p| p.import_assets(&paths.into_iter().map(PathBuf::from).collect::<Vec<_>>(), &subdir))
}

#[tauri::command]
pub fn asset_delete(state: State<AppState>, path: String) -> R<Vec<AssetInfo>> {
    with_project(&state, |p| {
        p.delete_asset(&path)?;
        p.assets()
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    html: String,
    headings: Vec<Heading>,
    warnings: Vec<String>,
    features: Vec<String>,
    words: usize,
    chapter_class: String,
}

#[tauri::command]
pub fn render_preview(state: State<AppState>, markdown: String, file: Option<String>) -> R<Preview> {
    with_project(&state, |p| {
        // Relative asset paths resolve against the preview's <base href>.
        let opts = RenderOptions { config: &p.config, asset_base: String::new(), chapter_file: file };
        let r = render_markdown(&markdown, &opts);
        Ok(Preview {
            html: r.html,
            headings: r.headings,
            warnings: r.warnings,
            features: r.features.into_iter().collect(),
            words: r.words,
            chapter_class: theme::chapter_classes(&p.config),
        })
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildArgs {
    targets: Vec<Target>,
    password: Option<String>,
}

#[tauri::command]
pub async fn build_book(app: AppHandle, state: State<'_, AppState>, args: BuildArgs) -> R<BuildReport> {
    let root = with_project(&state, |p| {
        p.save()?;
        Ok(p.root.clone())
    })?;
    let reader = state
        .resources
        .reader
        .clone()
        .ok_or("The reader template is missing. Run `bun run prepare` and restart Book Craft.")?;
    let req = BuildRequest {
        targets: args.targets,
        reader_template: reader,
        vendor_dir: state.resources.vendor.clone(),
        tools: state.tools(),
        certificate_password: args.password,
        keep_work_files: false,
    };
    tauri::async_runtime::spawn_blocking(move || {
        let project = Project::open(&root)?;
        package::build(&project, &req, &mut |line| {
            let _ = app.emit("build-log", line);
        })
    })
    .await
    .map_err(err)?
    .map_err(|e| format!("{e:#}"))
}

/// Opens the book in the real reader without packaging it.
#[tauri::command]
pub async fn test_read(state: State<'_, AppState>) -> R<()> {
    let root = with_project(&state, |p| {
        p.save()?;
        Ok(p.root.clone())
    })?;
    let reader = state.resources.reader.clone().ok_or("The reader template is missing. Run `bun run prepare`.")?;
    let vendor = state.resources.vendor.clone();
    tauri::async_runtime::spawn_blocking(move || -> anyhow::Result<()> {
        let project = Project::open(&root)?;
        let icon = icons::IconSet::load(&project)?;
        let dir = std::env::temp_dir().join("bookcraft").join("test-read");
        std::fs::create_dir_all(&dir)?;
        let pak_path = dir.join(format!("{}.pak", project.config.meta.package_id));
        pak::build(&project, vendor.as_deref(), &icon.png(256)?, &pak_path, &mut |_| {})?;
        std::process::Command::new(reader).arg("--book").arg(&pak_path).spawn()?;
        Ok(())
    })
    .await
    .map_err(err)?
    .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn reveal(path: String) -> R<()> {
    tauri_plugin_opener::reveal_item_in_dir(path).map_err(err)
}

/// Opens a built package (runs the exe, or hands the .msi/.msix to Windows).
#[tauri::command]
pub fn launch(path: String) -> R<()> {
    let p = PathBuf::from(&path);
    let ok = matches!(
        p.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
        Some("exe" | "msi" | "msix")
    );
    if !ok || !p.is_file() {
        return Err("Only built packages can be opened from here.".into());
    }
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(err)
}

#[tauri::command]
pub async fn certificate_inspect(path: String, password: Option<String>) -> R<CertificateInfo> {
    tauri::async_runtime::spawn_blocking(move || signing::inspect(std::path::Path::new(&path), password.as_deref()))
        .await
        .map_err(err)?
        .map_err(|e| format!("{e:#}"))
}

/// Makes a self-signed test certificate in the studio's own data folder (never in the book folder).
#[tauri::command]
pub async fn certificate_create(app: AppHandle, subject: String, password: String) -> R<TestCertificate> {
    let dir = app.path().app_data_dir().map_err(err)?.join("certificates");
    tauri::async_runtime::spawn_blocking(move || signing::create_test_certificate(&subject, &password, &dir))
        .await
        .map_err(err)?
        .map_err(|e| format!("{e:#}"))
}

/// Shows a Windows administrator prompt to trust a test certificate for MSIX installs.
#[tauri::command]
pub async fn certificate_trust(cer: String) -> R<()> {
    tauri::async_runtime::spawn_blocking(move || signing::trust_for_msix(std::path::Path::new(&cer)))
        .await
        .map_err(err)?
        .map_err(|e| format!("{e:#}"))
}
/// Moves the book's settings between book.json and book.toml.
#[tauri::command]
pub fn project_set_format(state: State<AppState>, format: ConfigFormat) -> R<ProjectData> {
    with_project(&state, |p| {
        p.set_format(format)?;
        Ok(project_data(p))
    })
}