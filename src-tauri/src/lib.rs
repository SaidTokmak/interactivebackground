mod commands;
mod desktop_host;
mod desktop_integration;
mod model;
mod monitors;
mod settings;
mod store;

use desktop_host::DesktopHostState;
use store::AppStore;
use tauri::Manager;
#[cfg(desktop)]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("interactivebackground")
                .arg("--hidden")
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Err(error) = commands::toggle_interaction_mode(app) {
                            eprintln!("Global kısayol çalıştırılamadı: {error}");
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let app_data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_directory)?;
            let database_path = app_data_directory.join("flowdesk.db");
            let store = AppStore::open(database_path).map_err(std::io::Error::other)?;
            let settings = store.get_settings().map_err(std::io::Error::other)?;
            let auto_calm_minutes = settings.auto_calm_minutes;
            app.manage(store);
            let desktop_host = DesktopHostState::default();
            desktop_host.configure_auto_calm(auto_calm_minutes);
            app.manage(desktop_host);
            app.global_shortcut()
                .register("Ctrl+Alt+Space")
                .map_err(std::io::Error::other)?;
            desktop_integration::setup_tray(app, settings.language)?;
            desktop_integration::setup_desktop_recovery(app.handle().clone());
            if std::env::args().any(|argument| argument == "--hidden") {
                if let Some(control) = app.get_webview_window("control") {
                    control.hide()?;
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    "control" => {
                        // Yönetim penceresinin X'i her durumda taskbar'dan kalkar
                        // ve sistem tepsisinden yeniden açılabilir.
                        api.prevent_close();
                        if let Err(error) = window.hide() {
                            eprintln!("Yönetim penceresi tepsiye gizlenemedi: {error}");
                        }
                    }
                    "wallpaper" => {
                        // Native X oluşsa bile kapanışı Rust yaşam döngüsüne
                        // yönlendirip WorkerW state'iyle birlikte temizleriz.
                        api.prevent_close();
                        if let Err(error) = commands::close_wallpaper_window(window.app_handle()) {
                            eprintln!("Wallpaper penceresi kapatılamadı: {error}");
                            if let Err(hide_error) = window.hide() {
                                eprintln!("Wallpaper zorunlu olarak gizlenemedi: {hide_error}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_tasks,
            commands::create_task,
            commands::toggle_task,
            commands::delete_task,
            commands::move_task,
            commands::get_settings,
            commands::update_settings,
            commands::get_background_settings,
            commands::update_background_settings,
            commands::choose_background_image,
            commands::get_widget_layout,
            commands::update_widget_layout,
            commands::reset_widget_layout,
            commands::list_monitors,
            commands::show_wallpaper,
            commands::hide_wallpaper,
            commands::desktop_host_status,
            commands::record_interaction_activity,
        ])
        .build(tauri::generate_context!())
        .expect("interactivebackground oluşturulurken beklenmeyen bir hata oluştu")
        .run(|app, event| {
            if let tauri::RunEvent::Ready = event {
                let marker = match app.path().app_data_dir() {
                    Ok(directory) => directory.join("recover-wallpaper"),
                    Err(error) => {
                        eprintln!("Kurtarma dizini okunamadı: {error}");
                        return;
                    }
                };
                if marker.exists() {
                    match commands::restore_wallpaper_after_restart(app) {
                        Ok(()) => {
                            if let Err(error) = std::fs::remove_file(marker) {
                                eprintln!("Kurtarma işareti silinemedi: {error}");
                            }
                        }
                        Err(error) => {
                            eprintln!("Wallpaper Ready aşamasında kurtarılamadı: {error}");
                        }
                    }
                }
            }
        });
}
