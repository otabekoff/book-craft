//! Studio state: the open project and the user's studio settings.

use std::path::PathBuf;
use std::sync::Mutex;

use book_core::Project;
use book_core::tools::ToolPaths;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StudioSettings {
    pub recent: Vec<String>,
    pub tools: ToolPaths,
    pub theme: String,
}

pub struct AppState {
    pub project: Mutex<Option<Project>>,
    pub settings: Mutex<StudioSettings>,
    settings_file: Option<PathBuf>,
    pub resources: Resources,
}

pub struct Resources {
    pub reader: Option<PathBuf>,
    pub vendor: Option<PathBuf>,
    pub extra_tool_dirs: Vec<PathBuf>,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Self {
        let settings_file = app.path().app_config_dir().ok().map(|d| d.join("settings.json"));
        let settings = settings_file
            .as_ref()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        Self {
            project: Mutex::new(None),
            settings: Mutex::new(settings),
            settings_file,
            resources: Resources::find(app),
        }
    }

    pub fn save_settings(&self) {
        let Some(file) = &self.settings_file else { return };
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(&*self.settings.lock().unwrap()) {
            let _ = std::fs::write(file, text);
        }
    }

    pub fn remember(&self, root: &std::path::Path) {
        let path = root.display().to_string();
        {
            let mut s = self.settings.lock().unwrap();
            s.recent.retain(|p| p != &path);
            s.recent.insert(0, path);
            s.recent.truncate(12);
        }
        self.save_settings();
    }

    pub fn tools(&self) -> ToolPaths {
        let mut t = self.settings.lock().unwrap().tools.clone();
        t.extra_dirs = self.resources.extra_tool_dirs.clone();
        t
    }
}

impl Resources {
    /// Bundled files live in the resource folder; during development they are
    /// read from `src-tauri/resources` (filled by `bun run prepare`).
    fn find(app: &AppHandle) -> Self {
        let mut roots = Vec::new();
        if let Ok(dir) = app.path().resource_dir() {
            roots.push(dir.join("resources"));
            roots.push(dir);
        }
        roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"));
        let pick = |rel: &str| roots.iter().map(|r| r.join(rel)).find(|p| p.exists());
        Self {
            reader: pick("reader/book-reader.exe"),
            vendor: pick("runtime/vendor"),
            extra_tool_dirs: roots.iter().map(|r| r.join("tools")).filter(|p| p.exists()).collect(),
        }
    }
}
