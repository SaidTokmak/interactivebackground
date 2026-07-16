use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{
    settings::{LanguagePreference, PomodoroCompletion, PomodoroMode},
    store::AppStore,
};

pub fn start(app: AppHandle) {
    std::thread::Builder::new()
        .name("pomodoro-monitor".into())
        .spawn(move || {
            loop {
                check_expired(&app);
                std::thread::sleep(Duration::from_millis(500));
            }
        })
        .expect("Pomodoro izleme iş parçacığı başlatılamadı");
}

fn check_expired(app: &AppHandle) {
    let store = app.state::<AppStore>();
    let completions = match store.complete_expired_pomodoros() {
        Ok(completions) => completions,
        Err(error) => {
            eprintln!("Pomodoro süreleri kontrol edilemedi: {error}");
            return;
        }
    };
    if completions.is_empty() {
        return;
    }

    let preferences = store.get_pomodoro_preferences().ok();
    let language = store
        .get_settings()
        .map(|settings| settings.language)
        .unwrap_or(LanguagePreference::System);
    for completion in completions {
        if preferences.is_some_and(|value| value.notifications_enabled) {
            show_notification(app, &completion, language);
        }
        if let Err(error) = app.emit("pomodoro-completed", &completion) {
            eprintln!("pomodoro-completed olayı yayınlanamadı: {error}");
        }
    }
    if let Err(error) = app.emit("pomodoro-changed", ()) {
        eprintln!("pomodoro-changed olayı yayınlanamadı: {error}");
    }
}

fn show_notification(
    app: &AppHandle,
    completion: &PomodoroCompletion,
    language: LanguagePreference,
) {
    let turkish = matches!(language, LanguagePreference::Tr);
    let (title, body) = match (turkish, completion.completed_mode) {
        (true, PomodoroMode::Work) => ("Pomodoro tamamlandı", "Odak seansı bitti. Mola zamanı."),
        (true, PomodoroMode::Break) => ("Mola tamamlandı", "Yeni odak seansına hazırsın."),
        (false, PomodoroMode::Work) => (
            "Pomodoro complete",
            "Focus session finished. Time for a break.",
        ),
        (false, PomodoroMode::Break) => ("Break complete", "Ready for another focus session."),
    };
    if let Err(error) = app
        .notification()
        .builder()
        .id(completion.widget_id as i32)
        .title(title)
        .body(body)
        .show()
    {
        eprintln!("Pomodoro bildirimi gösterilemedi: {error}");
    }
}
