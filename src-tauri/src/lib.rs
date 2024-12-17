use tauri::window::Color;
use tauri::{Position, WebviewWindow, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window =
                WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                    .accept_first_mouse(true)
                    .always_on_top(true)
                    .background_color(Color(0, 0, 0, 0))
                    .decorations(false)
                    .disable_drag_drop_handler()
                    .transparent(true)
                    .maximized(true)
                    .build()?;

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
