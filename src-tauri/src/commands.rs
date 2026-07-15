use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::DialogExt;

use crate::{
    desktop_host::{DesktopHostState, DesktopHostStatus},
    model::{Task, TaskStatus},
    monitors::MonitorInfo,
    settings::{AppSettings, BackgroundSettings, BackgroundSource},
    store::AppStore,
};

#[tauri::command]
pub fn list_tasks(store: State<'_, AppStore>) -> Result<Vec<Task>, String> {
    store.list()
}

#[tauri::command]
pub fn create_task(
    title: String,
    scheduled_for: Option<String>,
    store: State<'_, AppStore>,
    app: AppHandle,
) -> Result<Task, String> {
    let task = store.create(title, scheduled_for)?;
    notify_task_change(&app);
    Ok(task)
}

#[tauri::command]
pub fn toggle_task(id: i64, store: State<'_, AppStore>, app: AppHandle) -> Result<Task, String> {
    let task = store.toggle(id)?;
    notify_task_change(&app);
    Ok(task)
}

#[tauri::command]
pub fn move_task(
    id: i64,
    status: TaskStatus,
    store: State<'_, AppStore>,
    app: AppHandle,
) -> Result<Task, String> {
    let task = store.move_to(id, status)?;
    notify_task_change(&app);
    Ok(task)
}

#[tauri::command]
pub fn delete_task(id: i64, store: State<'_, AppStore>, app: AppHandle) -> Result<(), String> {
    store.delete(id)?;
    notify_task_change(&app);
    Ok(())
}

#[tauri::command]
pub fn get_settings(store: State<'_, AppStore>) -> Result<AppSettings, String> {
    store.get_settings()
}

#[tauri::command]
pub fn update_settings(
    settings: AppSettings,
    store: State<'_, AppStore>,
    desktop_host: State<'_, DesktopHostState>,
    app: AppHandle,
) -> Result<AppSettings, String> {
    let previous = store.get_settings()?;
    let settings = store.update_settings(settings)?;
    desktop_host.configure_auto_calm(settings.auto_calm_minutes);
    if previous.language != settings.language {
        if let Err(error) =
            crate::desktop_integration::update_tray_language(&app, settings.language)
        {
            eprintln!("Sistem tepsisi dili güncellenemedi: {error}");
        }
    }
    notify_settings_change(&app);
    // Ayar veritabanına başarıyla yazıldı. Anlık pencere taşıma ikincil bir yan
    // etkidir; başarısız olması kalıcı kaydı başarısız gibi göstermemelidir.
    let wallpaper_is_visible = app
        .get_webview_window("wallpaper")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    let native_mode_changed = previous.edit_mode != settings.edit_mode;
    let monitor_changed = previous.monitor_id != settings.monitor_id;

    if wallpaper_is_visible && (native_mode_changed || monitor_changed) {
        let result = if settings.edit_mode {
            desktop_host.enter_interaction_mode(&app, settings.monitor_id.as_deref())
        } else {
            activate_wallpaper_mode(&app, &desktop_host, settings.monitor_id.as_deref()).map(|_| ())
        };
        if let Err(error) = result {
            eprintln!("Wallpaper native modu güncellenemedi: {error}");
        }
        notify_desktop_host_change(&app);
    } else if !wallpaper_is_visible && monitor_changed {
        if let Err(error) =
            crate::monitors::position_wallpaper(&app, settings.monitor_id.as_deref())
        {
            eprintln!("Wallpaper yeni monitöre taşınamadı: {error}");
        }
    }
    Ok(settings)
}

#[tauri::command]
pub fn get_background_settings(
    monitor_id: Option<String>,
    store: State<'_, AppStore>,
    app: AppHandle,
) -> Result<BackgroundSettings, String> {
    let mut settings = store.get_background_settings(monitor_id)?;
    if settings.source == BackgroundSource::Custom
        && settings
            .custom_path
            .as_deref()
            .is_none_or(|path| !Path::new(path).is_file())
    {
        settings.source = BackgroundSource::Preset;
        settings.custom_path = None;
        settings = store.update_background_settings(settings)?;
        notify_background_change(&app);
    }
    Ok(settings)
}

#[tauri::command]
pub fn update_background_settings(
    settings: BackgroundSettings,
    store: State<'_, AppStore>,
    app: AppHandle,
) -> Result<BackgroundSettings, String> {
    if settings.source == BackgroundSource::Custom {
        let path = settings
            .custom_path
            .as_deref()
            .ok_or_else(|| "Özel arka plan dosyası bulunamadı.".to_string())?;
        validate_managed_background_path(&app, path)?;
    }

    let previous = store.get_background_settings(settings.monitor_id.clone())?;
    let updated = store.update_background_settings(settings)?;
    if previous.source == BackgroundSource::Custom && previous.custom_path != updated.custom_path {
        if let Some(path) = previous.custom_path.as_deref() {
            remove_managed_background(&app, path);
        }
    }
    notify_background_change(&app);
    Ok(updated)
}

#[tauri::command]
pub async fn choose_background_image(
    filter_name: String,
    app: AppHandle,
) -> Result<Option<String>, String> {
    let selected = app
        .dialog()
        .file()
        .add_filter(filter_name, &["jpg", "jpeg", "png", "webp"])
        .blocking_pick_file();
    let Some(selected) = selected else {
        return Ok(None);
    };
    let source = selected
        .into_path()
        .map_err(|error| format!("Arka plan dosya yolu okunamadı: {error}"))?;
    let destination = import_background_image_to(&source, &managed_background_directory(&app)?)?;
    Ok(Some(destination.to_string_lossy().into_owned()))
}

fn import_background_image_to(source: &Path, backgrounds: &Path) -> Result<PathBuf, String> {
    let source = source
        .canonicalize()
        .map_err(|error| format!("Arka plan dosyası açılamadı: {error}"))?;
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "Arka plan dosya türü desteklenmiyor.".to_string())?;
    if !matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp") {
        return Err("Arka plan dosya türü desteklenmiyor.".into());
    }

    let metadata = source
        .metadata()
        .map_err(|error| format!("Arka plan dosyası okunamadı: {error}"))?;
    if metadata.len() == 0 || metadata.len() > 50 * 1024 * 1024 {
        return Err("Arka plan görseli boş olamaz ve 50 MB'ı geçemez.".into());
    }
    validate_image_signature(&source, &extension)?;

    std::fs::create_dir_all(backgrounds)
        .map_err(|error| format!("Arka plan klasörü oluşturulamadı: {error}"))?;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Sistem saati okunamadı: {error}"))?
        .as_nanos();
    let extension = if extension == "jpeg" {
        "jpg"
    } else {
        &extension
    };
    let destination = backgrounds.join(format!("background-{suffix}.{extension}"));
    let temporary = backgrounds.join(format!(".background-{suffix}.tmp"));
    std::fs::copy(&source, &temporary)
        .map_err(|error| format!("Arka plan görseli kopyalanamadı: {error}"))?;
    std::fs::rename(&temporary, &destination).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        format!("Arka plan görseli kaydedilemedi: {error}")
    })?;
    Ok(destination)
}

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    crate::monitors::list(&app)
}

#[tauri::command]
pub fn show_wallpaper(
    app: AppHandle,
    store: State<'_, AppStore>,
    desktop_host: State<'_, DesktopHostState>,
) -> Result<DesktopHostStatus, String> {
    show_wallpaper_inner(&app, &store, &desktop_host)
}

#[tauri::command]
pub fn hide_wallpaper(
    app: AppHandle,
    desktop_host: State<'_, DesktopHostState>,
) -> Result<(), String> {
    hide_wallpaper_inner(&app, &desktop_host)
}

pub fn show_control_window(app: &AppHandle) -> Result<(), String> {
    let control = app
        .get_webview_window("control")
        .ok_or_else(|| "Yönetim penceresi bulunamadı.".to_string())?;
    control.show().map_err(window_error)?;
    control.unminimize().map_err(window_error)?;
    control.set_focus().map_err(window_error)
}

pub fn toggle_wallpaper(app: &AppHandle) -> Result<(), String> {
    let store = app.state::<AppStore>();
    let desktop_host = app.state::<DesktopHostState>();
    let visible = app
        .get_webview_window("wallpaper")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);

    if visible {
        hide_wallpaper_inner(app, &desktop_host)
    } else {
        show_wallpaper_inner(app, &store, &desktop_host).map(|_| ())
    }
}

pub fn toggle_interaction_mode(app: &AppHandle) -> Result<(), String> {
    ensure_wallpaper_window(app)?;
    let store = app.state::<AppStore>();
    let desktop_host = app.state::<DesktopHostState>();
    let mut settings = store.get_settings()?;
    settings.edit_mode = !desktop_host.is_interaction_mode();
    let settings = store.update_settings(settings)?;
    notify_settings_change(app);

    if settings.edit_mode {
        desktop_host.enter_interaction_mode(app, settings.monitor_id.as_deref())?;
    } else {
        activate_wallpaper_mode(app, &desktop_host, settings.monitor_id.as_deref())?;
    }
    notify_desktop_host_change(app);
    Ok(())
}

pub fn quit_application(app: &AppHandle) {
    let desktop_host = app.state::<DesktopHostState>();
    if let Err(error) = desktop_host.force_hide_window(app) {
        eprintln!("Çıkış sırasında wallpaper gizlenemedi: {error}");
    }
    if let Err(error) = desktop_host.detach(app) {
        eprintln!("Çıkış sırasında WorkerW bağlantısı kaldırılamadı: {error}");
    }
    if let Err(error) = desktop_host.leave_interaction_mode(app) {
        eprintln!("Çıkış sırasında etkileşim modu temizlenemedi: {error}");
    }
    app.exit(0);
}

pub fn close_wallpaper_window(app: &AppHandle) -> Result<(), String> {
    let desktop_host = app.state::<DesktopHostState>();
    hide_wallpaper_inner(app, &desktop_host)
}

#[tauri::command]
pub fn record_interaction_activity(
    desktop_host: State<'_, DesktopHostState>,
) -> Result<(), String> {
    desktop_host.record_interaction_activity()
}

pub fn apply_auto_calm_if_due(app: &AppHandle) -> Result<bool, String> {
    let desktop_host = app.state::<DesktopHostState>();
    if !desktop_host.auto_calm_due() {
        return Ok(false);
    }

    let store = app.state::<AppStore>();
    let mut settings = store.get_settings()?;
    settings.edit_mode = false;
    let settings = store.update_settings(settings)?;
    notify_settings_change(app);
    activate_wallpaper_mode(app, &desktop_host, settings.monitor_id.as_deref())?;
    notify_desktop_host_change(app);
    eprintln!("interactivebackground otomatik olarak sakin moda döndü.");
    Ok(true)
}

pub fn recover_desktop_host(app: &AppHandle) -> Result<bool, String> {
    let desktop_host = app.state::<DesktopHostState>();
    if !desktop_host.recovery_needed() {
        return Ok(false);
    }

    let marker = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Kurtarma dizini bulunamadı: {error}"))?
        .join("recover-wallpaper");
    std::fs::write(&marker, b"workerw-recovery")
        .map_err(|error| format!("Kurtarma işareti yazılamadı: {error}"))?;
    eprintln!(
        "Explorer wallpaper HWND'sini yok etti; interactivebackground kurtarma için yeniden başlatılıyor."
    );
    app.request_restart();
    Ok(true)
}

pub fn restore_wallpaper_after_restart(app: &AppHandle) -> Result<(), String> {
    let store = app.state::<AppStore>();
    let desktop_host = app.state::<DesktopHostState>();
    show_wallpaper_inner(app, &store, &desktop_host)?;
    eprintln!("interactivebackground wallpaper yeniden başlatma sonrasında kurtarıldı.");
    Ok(())
}

fn show_wallpaper_inner(
    app: &AppHandle,
    store: &AppStore,
    desktop_host: &DesktopHostState,
) -> Result<DesktopHostStatus, String> {
    ensure_wallpaper_window(app)?;
    let settings = store.get_settings()?;
    let status = if settings.edit_mode {
        desktop_host.enter_interaction_mode(app, settings.monitor_id.as_deref())?;
        desktop_host.status(None)
    } else {
        activate_wallpaper_mode(app, desktop_host, settings.monitor_id.as_deref())?
    };
    notify_desktop_host_change(app);
    Ok(status)
}

fn hide_wallpaper_inner(app: &AppHandle, desktop_host: &DesktopHostState) -> Result<(), String> {
    show_control_window(app)?;
    desktop_host.destroy_wallpaper_window(app)?;

    notify_desktop_host_change(app);
    eprintln!("interactivebackground wallpaper penceresi tamamen kapatıldı.");

    Ok(())
}

fn ensure_wallpaper_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("wallpaper") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, "wallpaper", WebviewUrl::App("index.html".into()))
        .title("interactivebackground Wallpaper")
        .inner_size(1280.0, 720.0)
        .resizable(false)
        .decorations(false)
        .shadow(false)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .build()
        .map_err(window_error)
}

#[tauri::command]
pub fn desktop_host_status(desktop_host: State<'_, DesktopHostState>) -> DesktopHostStatus {
    desktop_host.status(None)
}

fn notify_task_change(app: &AppHandle) {
    // Veritabanı işlemi zaten tamamlandığı için event hatasını kullanıcıya CRUD
    // hatası gibi döndürmüyoruz. Kaynak pencere kendi state'ini yine günceller.
    if let Err(error) = app.emit("tasks-changed", ()) {
        eprintln!("tasks-changed olayı yayınlanamadı: {error}");
    }
}

fn notify_settings_change(app: &AppHandle) {
    if let Err(error) = app.emit("settings-changed", ()) {
        eprintln!("settings-changed olayı yayınlanamadı: {error}");
    }
}

fn notify_background_change(app: &AppHandle) {
    if let Err(error) = app.emit("background-settings-changed", ()) {
        eprintln!("background-settings-changed olayı yayınlanamadı: {error}");
    }
}

fn notify_desktop_host_change(app: &AppHandle) {
    if let Err(error) = app.emit("desktop-host-changed", ()) {
        eprintln!("desktop-host-changed olayı yayınlanamadı: {error}");
    }
}

fn activate_wallpaper_mode(
    app: &AppHandle,
    desktop_host: &DesktopHostState,
    monitor_id: Option<&str>,
) -> Result<DesktopHostStatus, String> {
    desktop_host.detach(app)?;
    desktop_host.leave_interaction_mode(app)?;
    crate::monitors::position_wallpaper(app, monitor_id)?;
    let wallpaper = app
        .get_webview_window("wallpaper")
        .ok_or_else(|| "Wallpaper penceresi bulunamadı.".to_string())?;
    wallpaper.show().map_err(window_error)?;
    // Attach başarısız olursa gösterilen normal pencere de aktif bir wallpaper
    // modudur. Başarılı attach bu bayrağı DesktopHostState içinde temizler.
    desktop_host.set_fallback_mode(true);

    // WorkerW belgelenmemiş bir Windows kabuk ayrıntısıdır. Bağlantı kurulamazsa
    // pencereyi kapatmayız; normal always-on-bottom yerleşimi çalışmaya devam eder.
    match desktop_host.attach(app, monitor_id) {
        Ok(()) => Ok(desktop_host.status(None)),
        Err(error) => {
            eprintln!("WorkerW bağlantısı kurulamadı, normal pencere kullanılıyor: {error}");
            Ok(desktop_host.status(Some(error)))
        }
    }
}

fn window_error(error: tauri::Error) -> String {
    format!("Pencere işlemi başarısız: {error}")
}

fn managed_background_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("backgrounds"))
        .map_err(|error| format!("Arka plan klasörü bulunamadı: {error}"))
}

fn validate_managed_background_path(app: &AppHandle, path: &str) -> Result<(), String> {
    let managed = managed_background_directory(app)?
        .canonicalize()
        .map_err(|error| format!("Arka plan klasörü okunamadı: {error}"))?;
    let candidate = PathBuf::from(path)
        .canonicalize()
        .map_err(|error| format!("Arka plan dosyası açılamadı: {error}"))?;
    if !candidate.starts_with(managed) || !candidate.is_file() {
        return Err("Özel arka plan uygulamanın yönetilen klasöründe değil.".into());
    }
    Ok(())
}

fn remove_managed_background(app: &AppHandle, path: &str) {
    let Ok(managed) = managed_background_directory(app).and_then(|path| {
        path.canonicalize()
            .map_err(|error| format!("Arka plan klasörü okunamadı: {error}"))
    }) else {
        return;
    };
    let Ok(candidate) = PathBuf::from(path).canonicalize() else {
        return;
    };
    if candidate.starts_with(managed) {
        if let Err(error) = std::fs::remove_file(candidate) {
            eprintln!("Eski arka plan görseli silinemedi: {error}");
        }
    }
}

fn validate_image_signature(path: &Path, extension: &str) -> Result<(), String> {
    let mut header = [0_u8; 12];
    let mut file =
        File::open(path).map_err(|error| format!("Arka plan dosyası okunamadı: {error}"))?;
    let bytes_read = file
        .read(&mut header)
        .map_err(|error| format!("Arka plan dosyası okunamadı: {error}"))?;
    let valid = match extension {
        "png" => bytes_read >= 8 && header[..8] == [137, 80, 78, 71, 13, 10, 26, 10],
        "jpg" | "jpeg" => bytes_read >= 3 && header[..3] == [0xff, 0xd8, 0xff],
        "webp" => bytes_read >= 12 && &header[..4] == b"RIFF" && &header[8..12] == b"WEBP",
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err("Arka plan dosyasının içeriği seçilen görsel türüyle eşleşmiyor.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::{import_background_image_to, validate_image_signature};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn accepts_matching_image_signatures_and_rejects_disguised_files() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let png = std::env::temp_dir().join(format!("interactivebackground-{suffix}.png"));
        let fake = std::env::temp_dir().join(format!("interactivebackground-{suffix}-fake.png"));
        std::fs::write(&png, [137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 0]).unwrap();
        std::fs::write(&fake, b"not an image").unwrap();

        assert!(validate_image_signature(&png, "png").is_ok());
        assert!(validate_image_signature(&fake, "png").is_err());

        std::fs::remove_file(png).unwrap();
        std::fs::remove_file(fake).unwrap();
    }

    #[test]
    fn imports_a_valid_image_into_the_managed_directory() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("interactivebackground-import-{suffix}"));
        let source = root.join("source.png");
        let managed = root.join("managed");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&source, [137, 80, 78, 71, 13, 10, 26, 10, 1, 2, 3, 4]).unwrap();

        let imported = import_background_image_to(&source, &managed).unwrap();
        assert!(imported.starts_with(&managed));
        assert_eq!(
            std::fs::read(imported).unwrap(),
            std::fs::read(source).unwrap()
        );

        std::fs::remove_dir_all(root).unwrap();
    }
}
