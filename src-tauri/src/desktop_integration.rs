use std::{thread, time::Duration};

use tauri::{
    App, AppHandle,
    menu::MenuBuilder,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};

const SHOW_CONTROL: &str = "show-control";
const TOGGLE_WALLPAPER: &str = "toggle-wallpaper";
const TOGGLE_INTERACTION: &str = "toggle-interaction";
const QUIT: &str = "quit-flowdesk";

pub fn setup_tray(app: &mut App) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text(SHOW_CONTROL, "Yönetim panelini aç")
        .text(TOGGLE_WALLPAPER, "Wallpaper'ı aç / kapat")
        .text(
            TOGGLE_INTERACTION,
            "Etkileşim modunu aç / kapat   Ctrl+Alt+Space",
        )
        .separator()
        .text(QUIT, "Flowdesk'ten çık")
        .build()?;

    let mut tray = TrayIconBuilder::with_id("flowdesk")
        .menu(&menu)
        .tooltip("Flowdesk")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let result = match event.id().as_ref() {
                SHOW_CONTROL => crate::commands::show_control_window(app),
                TOGGLE_WALLPAPER => crate::commands::toggle_wallpaper(app),
                TOGGLE_INTERACTION => crate::commands::toggle_interaction_mode(app),
                QUIT => {
                    crate::commands::quit_application(app);
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                eprintln!("Sistem tepsisi işlemi başarısız: {error}");
            }
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                if let Err(error) = crate::commands::show_control_window(tray.app_handle()) {
                    eprintln!("Yönetim penceresi tepsiden açılamadı: {error}");
                }
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }
    tray.build(app)?;
    Ok(())
}

pub fn setup_desktop_recovery(app: AppHandle) {
    thread::spawn(move || {
        let mut last_error: Option<String> = None;
        loop {
            thread::sleep(Duration::from_secs(3));
            match crate::commands::recover_desktop_host(&app) {
                Ok(true) => last_error = None,
                Ok(false) => {}
                Err(error) => {
                    if last_error.as_deref() != Some(error.as_str()) {
                        eprintln!("Explorer WorkerW kurtarma denemesi başarısız: {error}");
                        last_error = Some(error);
                    }
                }
            }
        }
    });
}
