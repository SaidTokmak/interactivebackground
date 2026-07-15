use std::{thread, time::Duration};

use tauri::{
    App, AppHandle, Runtime,
    menu::{Menu, MenuBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};

use crate::settings::LanguagePreference;

const SHOW_CONTROL: &str = "show-control";
const TOGGLE_WALLPAPER: &str = "toggle-wallpaper";
const TOGGLE_INTERACTION: &str = "toggle-interaction";
const QUIT: &str = "quit-interactivebackground";

pub fn setup_tray(app: &mut App, language: LanguagePreference) -> tauri::Result<()> {
    let menu = build_tray_menu(app.handle(), language)?;

    let mut tray = TrayIconBuilder::with_id("interactivebackground")
        .menu(&menu)
        .tooltip("interactivebackground")
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

pub fn update_tray_language(app: &AppHandle, language: LanguagePreference) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("interactivebackground") {
        tray.set_menu(Some(build_tray_menu(app, language)?))?;
    }
    Ok(())
}

fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    language: LanguagePreference,
) -> tauri::Result<Menu<R>> {
    let labels = tray_labels(resolve_language(language));
    MenuBuilder::new(app)
        .text(SHOW_CONTROL, labels.show_control)
        .text(TOGGLE_WALLPAPER, labels.toggle_wallpaper)
        .text(TOGGLE_INTERACTION, labels.toggle_interaction)
        .separator()
        .text(QUIT, labels.quit)
        .build()
}

#[derive(Clone, Copy)]
enum ResolvedLanguage {
    Tr,
    En,
}

struct TrayLabels {
    show_control: &'static str,
    toggle_wallpaper: &'static str,
    toggle_interaction: &'static str,
    quit: &'static str,
}

fn tray_labels(language: ResolvedLanguage) -> TrayLabels {
    match language {
        ResolvedLanguage::Tr => TrayLabels {
            show_control: "Yönetim panelini aç",
            toggle_wallpaper: "Wallpaper'ı aç / kapat",
            toggle_interaction: "Etkileşim modunu aç / kapat   Ctrl+Alt+Space",
            quit: "interactivebackground'dan çık",
        },
        ResolvedLanguage::En => TrayLabels {
            show_control: "Open control panel",
            toggle_wallpaper: "Show / hide wallpaper",
            toggle_interaction: "Toggle interaction mode   Ctrl+Alt+Space",
            quit: "Quit interactivebackground",
        },
    }
}

fn resolve_language(preference: LanguagePreference) -> ResolvedLanguage {
    match preference {
        LanguagePreference::Tr => ResolvedLanguage::Tr,
        LanguagePreference::En => ResolvedLanguage::En,
        LanguagePreference::System => system_language(),
    }
}

#[cfg(target_os = "windows")]
fn system_language() -> ResolvedLanguage {
    use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;

    let mut locale_name = [0_u16; 85];
    // Windows API null sonlandırıcı dahil yazılan karakter sayısını döndürür.
    let length =
        unsafe { GetUserDefaultLocaleName(locale_name.as_mut_ptr(), locale_name.len() as i32) };
    if length > 1 {
        let locale = String::from_utf16_lossy(&locale_name[..(length as usize - 1)]);
        if locale.to_ascii_lowercase().starts_with("tr") {
            return ResolvedLanguage::Tr;
        }
    }
    ResolvedLanguage::En
}

#[cfg(not(target_os = "windows"))]
fn system_language() -> ResolvedLanguage {
    if std::env::var("LANG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .starts_with("tr")
    {
        ResolvedLanguage::Tr
    } else {
        ResolvedLanguage::En
    }
}

pub fn setup_desktop_recovery(app: AppHandle) {
    thread::spawn(move || {
        let mut last_error: Option<String> = None;
        loop {
            thread::sleep(Duration::from_secs(1));
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
            if let Err(error) = crate::commands::apply_auto_calm_if_due(&app) {
                eprintln!("Otomatik sakin moda geçilemedi: {error}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{ResolvedLanguage, tray_labels};

    #[test]
    fn provides_turkish_and_english_tray_labels() {
        let tr = tray_labels(ResolvedLanguage::Tr);
        let en = tray_labels(ResolvedLanguage::En);

        assert_eq!(tr.show_control, "Yönetim panelini aç");
        assert_eq!(tr.quit, "interactivebackground'dan çık");
        assert_eq!(en.show_control, "Open control panel");
        assert_eq!(en.quit, "Quit interactivebackground");
    }
}
