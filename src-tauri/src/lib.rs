//! Book Craft studio: the Tauri backend for the authoring app.

mod commands;
mod preview;
mod state;

use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("bookasset", |ctx, req| preview::serve(ctx.app_handle(), req))
        .setup(|app| {
            let state = state::AppState::load(app.handle());
            app.manage(state);
            // Ctrl+C in the terminal (or closing it) would otherwise kill the app mid-edit.
            // Ask the window to save, then exit; quit anyway if it does not answer in time.
            let handle = app.handle().clone();
            let _ = ctrlc::set_handler(move || {
                let _ = handle.emit("app-exit-requested", ());
                let h = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    h.exit(0);
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::save_settings,
            commands::project_create,
            commands::project_open,
            commands::project_close,
            commands::project_save_config,
            commands::project_validate,
            commands::project_set_format,
            commands::chapter_read,
            commands::chapter_write,
            commands::chapter_add,
            commands::chapter_remove,
            commands::assets_list,
            commands::assets_import,
            commands::asset_delete,
            commands::render_preview,
            commands::build_book,
            commands::test_read,
            commands::reveal,
            commands::launch,
            commands::quit,
            commands::certificate_inspect,
            commands::certificate_create,
            commands::certificate_trust,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Book Craft");
}
