use serde::Serialize;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, window::Monitor};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn list(app: &AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let window = app
        .get_webview_window("control")
        .or_else(|| app.get_webview_window("wallpaper"))
        .ok_or_else(|| "Monitörleri okuyacak bir pencere bulunamadı.".to_string())?;
    let monitors = window.available_monitors().map_err(monitor_error)?;
    let primary_id = window
        .primary_monitor()
        .map_err(monitor_error)?
        .as_ref()
        .map(monitor_id);

    Ok(monitors
        .iter()
        .enumerate()
        .map(|(index, monitor)| monitor_info(monitor, index, primary_id.as_deref()))
        .collect())
}

pub fn position_wallpaper(app: &AppHandle, selected_id: Option<&str>) -> Result<(), String> {
    let wallpaper = app
        .get_webview_window("wallpaper")
        .ok_or_else(|| "Wallpaper penceresi bulunamadı.".to_string())?;
    let selected = selected_monitor(&wallpaper, selected_id)?;

    // Native fullscreen bazı platformlarda ayrı bir çalışma alanı oluşturabilir.
    // WorkerW öncesi pencereyi fiziksel monitör sınırlarına kendimiz yerleştiririz.
    wallpaper.set_fullscreen(false).map_err(monitor_error)?;
    wallpaper.set_decorations(false).map_err(monitor_error)?;
    wallpaper.set_shadow(false).map_err(monitor_error)?;
    wallpaper.set_resizable(false).map_err(monitor_error)?;
    wallpaper
        .set_size(PhysicalSize::new(
            selected.size().width,
            selected.size().height,
        ))
        .map_err(monitor_error)?;
    wallpaper
        .set_always_on_bottom(true)
        .map_err(monitor_error)?;

    let target_position = PhysicalPosition::new(selected.position().x, selected.position().y);
    wallpaper
        .set_position(target_position)
        .map_err(monitor_error)?;

    // Windows DWM, borderless pencerede görünmez bir dış sınır bırakabilir.
    // Kullanıcının gördüğü WebView içeriğini monitör köşesine hizalamak için
    // inner/outer farkını ölçüp dış pozisyonu bir kez telafi ederiz.
    let inner_position = wallpaper.inner_position().map_err(monitor_error)?;
    if inner_position != target_position {
        let outer_position = wallpaper.outer_position().map_err(monitor_error)?;
        wallpaper
            .set_position(PhysicalPosition::new(
                outer_position.x + target_position.x - inner_position.x,
                outer_position.y + target_position.y - inner_position.y,
            ))
            .map_err(monitor_error)?;
    }

    let final_inner = wallpaper.inner_position().map_err(monitor_error)?;
    let final_size = wallpaper.inner_size().map_err(monitor_error)?;
    eprintln!(
        "Flowdesk wallpaper yerleşimi: hedef=({}, {}) {}x{}, gerçek=({}, {}) {}x{}",
        target_position.x,
        target_position.y,
        selected.size().width,
        selected.size().height,
        final_inner.x,
        final_inner.y,
        final_size.width,
        final_size.height,
    );
    Ok(())
}

pub fn selected_bounds(
    app: &AppHandle,
    selected_id: Option<&str>,
) -> Result<MonitorBounds, String> {
    let wallpaper = app
        .get_webview_window("wallpaper")
        .ok_or_else(|| "Wallpaper penceresi bulunamadı.".to_string())?;
    let selected = selected_monitor(&wallpaper, selected_id)?;

    Ok(MonitorBounds {
        x: selected.position().x,
        y: selected.position().y,
        width: selected.size().width,
        height: selected.size().height,
    })
}

fn selected_monitor(
    wallpaper: &tauri::WebviewWindow,
    selected_id: Option<&str>,
) -> Result<Monitor, String> {
    let monitors = wallpaper.available_monitors().map_err(monitor_error)?;
    selected_id
        .and_then(|id| {
            monitors
                .iter()
                .find(|monitor| monitor_id(monitor) == id)
                .cloned()
        })
        .or_else(|| wallpaper.primary_monitor().ok().flatten())
        .or_else(|| monitors.into_iter().next())
        .ok_or_else(|| "Kullanılabilir monitör bulunamadı.".to_string())
}

fn monitor_info(monitor: &Monitor, index: usize, primary_id: Option<&str>) -> MonitorInfo {
    let id = monitor_id(monitor);
    MonitorInfo {
        name: monitor
            .name()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("Ekran {}", index + 1)),
        x: monitor.position().x,
        y: monitor.position().y,
        width: monitor.size().width,
        height: monitor.size().height,
        scale_factor: monitor.scale_factor(),
        is_primary: primary_id == Some(id.as_str()),
        id,
    }
}

fn monitor_id(monitor: &Monitor) -> String {
    format!(
        "{}:{}:{}:{}x{}",
        monitor.name().map(String::as_str).unwrap_or("monitor"),
        monitor.position().x,
        monitor.position().y,
        monitor.size().width,
        monitor.size().height,
    )
}

fn monitor_error(error: tauri::Error) -> String {
    format!("Monitör işlemi başarısız: {error}")
}
