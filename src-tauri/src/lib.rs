//! Book Craft studio: the Tauri backend for the authoring app.

mod commands;
mod preview;
mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("bookasset", |ctx, req| preview::serve(ctx.app_handle(), req))
        .setup(|app| {
            let state = state::AppState::load(app.handle());
            app.manage(state);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Book Craft");
}
