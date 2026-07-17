use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopHostStatus {
    pub attached: bool,
    pub visible: bool,
    pub closing: bool,
    pub mode: &'static str,
    pub warning: Option<String>,
}

/// WorkerW bağlantısının geri alınabilmesi için native pencerenin önceki
/// parent/style değerlerini saklar. Native handle'ları pointer olarak değil
/// `isize` olarak tutmak state'in thread-safe (`Send + Sync`) kalmasını sağlar.
#[derive(Clone, Copy, Debug)]
struct Attachment {
    window: isize,
    worker: isize,
    original_parent: isize,
    original_style: isize,
}

#[derive(Default)]
pub struct DesktopHostState {
    attachment: Mutex<Option<Attachment>>,
    interaction_mode: AtomicBool,
    fallback_mode: AtomicBool,
    workerw_desired: AtomicBool,
    wallpaper_desired: AtomicBool,
    wallpaper_visible: AtomicBool,
    wallpaper_closing: AtomicBool,
    auto_calm_minutes: AtomicU32,
    last_interaction_activity: Mutex<Option<Instant>>,
}

impl DesktopHostState {
    pub fn refresh_desktop(&self) -> Result<(), String> {
        platform::refresh_desktop()
    }

    pub fn force_hide_window(&self, app: &AppHandle) -> Result<(), String> {
        let wallpaper = wallpaper_window(app)?;
        wallpaper.hide().map_err(window_error)?;
        platform::force_hide(app)
    }

    pub fn attach(&self, app: &AppHandle, monitor_id: Option<&str>) -> Result<(), String> {
        self.workerw_desired.store(true, Ordering::Release);
        let mut attachment = self.lock()?;
        if attachment.is_some() {
            return Ok(());
        }

        let wallpaper = wallpaper_window(app)?;
        wallpaper.set_always_on_top(false).map_err(window_error)?;
        wallpaper.set_always_on_bottom(true).map_err(window_error)?;
        *attachment = Some(platform::attach(app, monitor_id)?);
        self.interaction_mode.store(false, Ordering::Release);
        self.fallback_mode.store(false, Ordering::Release);
        eprintln!("interactivebackground wallpaper WorkerW masaüstü katmanına bağlandı.");
        Ok(())
    }

    pub fn detach(&self, app: &AppHandle) -> Result<(), String> {
        self.workerw_desired.store(false, Ordering::Release);
        let mut attachment = self.lock()?;
        let Some(current) = *attachment else {
            return Ok(());
        };

        platform::detach(app, current)?;
        *attachment = None;
        eprintln!("interactivebackground wallpaper WorkerW masaüstü katmanından ayrıldı.");
        Ok(())
    }

    pub fn is_attached(&self) -> bool {
        self.lock().map(|value| value.is_some()).unwrap_or(false)
    }

    pub fn recovery_needed(&self) -> bool {
        if !self.workerw_desired.load(Ordering::Acquire) {
            return false;
        }
        self.lock()
            .map(|attachment| {
                attachment
                    .as_ref()
                    .is_none_or(|current| !platform::attachment_is_valid(*current))
            })
            .unwrap_or(false)
    }

    pub fn enter_interaction_mode(
        &self,
        app: &AppHandle,
        monitor_id: Option<&str>,
    ) -> Result<(), String> {
        self.detach(app)?;
        crate::monitors::position_wallpaper(app, monitor_id)?;

        let wallpaper = wallpaper_window(app)?;
        // Etkileşim katmanı normal bir top-level penceredir. WorkerW'den farklı
        // olarak Explorer'ın ikon katmanının üstünde durur ve WebView fare/klavye
        // olaylarını doğrudan alabilir.
        wallpaper
            .set_always_on_bottom(false)
            .map_err(window_error)?;
        wallpaper.set_always_on_top(true).map_err(window_error)?;
        wallpaper.show().map_err(window_error)?;
        wallpaper.set_focus().map_err(window_error)?;
        self.interaction_mode.store(true, Ordering::Release);
        self.fallback_mode.store(false, Ordering::Release);
        *self.activity_lock()? = Some(Instant::now());
        eprintln!("interactivebackground tıklanabilir etkileşim katmanına geçti.");
        Ok(())
    }

    pub fn leave_interaction_mode(&self, app: &AppHandle) -> Result<(), String> {
        let wallpaper = wallpaper_window(app)?;
        wallpaper.set_always_on_top(false).map_err(window_error)?;
        self.interaction_mode.store(false, Ordering::Release);
        *self.activity_lock()? = None;
        Ok(())
    }

    pub fn is_interaction_mode(&self) -> bool {
        self.interaction_mode.load(Ordering::Acquire)
    }

    pub fn request_wallpaper_visibility(&self, visible: bool) {
        self.wallpaper_desired.store(visible, Ordering::Release);
        if !visible {
            self.wallpaper_visible.store(false, Ordering::Release);
            self.fallback_mode.store(false, Ordering::Release);
        }
    }

    pub fn confirm_wallpaper_visible(&self) {
        self.wallpaper_visible.store(true, Ordering::Release);
        self.wallpaper_closing.store(false, Ordering::Release);
    }

    pub fn begin_wallpaper_close(&self) {
        self.wallpaper_closing.store(true, Ordering::Release);
    }

    pub fn finish_wallpaper_close(&self) {
        self.wallpaper_closing.store(false, Ordering::Release);
    }

    pub fn wallpaper_is_closing(&self) -> bool {
        self.wallpaper_closing.load(Ordering::Acquire)
    }

    pub fn wallpaper_is_desired(&self) -> bool {
        self.wallpaper_desired.load(Ordering::Acquire)
    }

    pub fn wallpaper_is_visible(&self) -> bool {
        self.wallpaper_visible.load(Ordering::Acquire)
    }

    pub fn configure_auto_calm(&self, minutes: Option<u16>) {
        self.auto_calm_minutes
            .store(minutes.map(u32::from).unwrap_or(0), Ordering::Release);
    }

    pub fn record_interaction_activity(&self) -> Result<(), String> {
        if self.is_interaction_mode() {
            *self.activity_lock()? = Some(Instant::now());
        }
        Ok(())
    }

    pub fn auto_calm_due(&self) -> bool {
        let minutes = self.auto_calm_minutes.load(Ordering::Acquire);
        if minutes == 0 || !self.is_interaction_mode() {
            return false;
        }
        self.activity_lock()
            .ok()
            .and_then(|activity| *activity)
            .is_some_and(|last| last.elapsed() >= Duration::from_secs(u64::from(minutes) * 60))
    }

    pub fn set_fallback_mode(&self, active: bool) {
        self.fallback_mode.store(active, Ordering::Release);
    }

    pub fn status(&self, warning: Option<String>) -> DesktopHostStatus {
        let attached = self.is_attached();
        let visible = self.wallpaper_is_visible();
        let interaction = self.is_interaction_mode();
        let fallback = self.fallback_mode.load(Ordering::Acquire);
        DesktopHostStatus {
            attached,
            visible,
            closing: self.wallpaper_is_closing(),
            mode: if !visible {
                "window"
            } else if attached {
                "workerW"
            } else if interaction {
                "interaction"
            } else if fallback {
                "fallback"
            } else {
                "window"
            },
            warning,
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Option<Attachment>>, String> {
        self.attachment
            .lock()
            .map_err(|_| "Masaüstü bağlantı durumu kilitlenemedi.".to_string())
    }

    fn activity_lock(&self) -> Result<std::sync::MutexGuard<'_, Option<Instant>>, String> {
        self.last_interaction_activity
            .lock()
            .map_err(|_| "Etkileşim süresi kilitlenemedi.".to_string())
    }
}

fn wallpaper_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window("wallpaper")
        .ok_or_else(|| "Wallpaper penceresi bulunamadı.".to_string())
}

fn window_error(error: tauri::Error) -> String {
    format!("Masaüstü pencere modu değiştirilemedi: {error}")
}

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr;

    use tauri::Manager;
    use windows::{
        Win32::{
            System::Com::{
                CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoTaskMemFree, CoUninitialize,
            },
            UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
        },
        core::HSTRING,
    };
    use windows_sys::{
        Win32::{
            Foundation::{GetLastError, HWND, LPARAM, RECT, SetLastError, WPARAM},
            UI::WindowsAndMessaging::{
                EnumWindows, FindWindowExW, FindWindowW, GWL_STYLE, GetParent, GetWindowLongPtrW,
                GetWindowRect, IsWindow, IsWindowVisible, SMTO_NORMAL, SW_HIDE, SWP_FRAMECHANGED,
                SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SendMessageTimeoutW,
                SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, WS_CHILD, WS_POPUP,
            },
        },
        core::BOOL,
    };

    use super::Attachment;

    const SPAWN_WORKERW_MESSAGE: u32 = 0x052C;

    pub fn refresh_desktop() -> Result<(), String> {
        // Explorer bazen yok edilen WorkerW child'ının son karesini özellikle
        // ikincil monitörde önbellekte tutar. Monitörlerin mevcut duvar kâğıdını
        // yine kendisine uygulamak bu katmanı kullanıcı ayarını değiştirmeden
        // yeniden oluşturmaya zorlar.
        std::thread::spawn(refresh_desktop_sta)
            .join()
            .map_err(|_| "Masaüstü yenileme iş parçacığı sonlandırıldı.".to_string())?
    }

    fn refresh_desktop_sta() -> Result<(), String> {
        // SAFETY: Bu iş parçacığı yalnızca bu işlem için oluşturulur; COM aynı
        // iş parçacığında başlatılıp her çıkış yolunda kapatılır.
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(|error| format!("Masaüstü COM katmanı başlatılamadı: {error}"))?;

            let result = (|| {
                let desktop: IDesktopWallpaper =
                    CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL).map_err(|error| {
                        format!("Windows duvar kâğıdı servisi açılamadı: {error}")
                    })?;
                let count = desktop
                    .GetMonitorDevicePathCount()
                    .map_err(|error| format!("Monitör sayısı okunamadı: {error}"))?;

                for index in 0..count {
                    let monitor_ptr = desktop
                        .GetMonitorDevicePathAt(index)
                        .map_err(|error| format!("Monitör kimliği okunamadı: {error}"))?;
                    let monitor = monitor_ptr
                        .to_string()
                        .map_err(|error| format!("Monitör kimliği dönüştürülemedi: {error}"))?;
                    CoTaskMemFree(Some(monitor_ptr.0.cast()));

                    let monitor = HSTRING::from(monitor);
                    let wallpaper_ptr = desktop
                        .GetWallpaper(&monitor)
                        .map_err(|error| format!("Duvar kâğıdı yolu okunamadı: {error}"))?;
                    let wallpaper = wallpaper_ptr
                        .to_string()
                        .map_err(|error| format!("Duvar kâğıdı yolu dönüştürülemedi: {error}"))?;
                    CoTaskMemFree(Some(wallpaper_ptr.0.cast()));

                    if !wallpaper.is_empty() {
                        desktop
                            .SetWallpaper(&monitor, &HSTRING::from(wallpaper))
                            .map_err(|error| format!("Masaüstü yenilenemedi: {error}"))?;
                    }
                }
                Ok(())
            })();

            CoUninitialize();
            result
        }
    }

    pub fn force_hide(app: &tauri::AppHandle) -> Result<(), String> {
        let wallpaper = app
            .get_webview_window("wallpaper")
            .ok_or_else(|| "Wallpaper penceresi bulunamadı.".to_string())?;
        let window = wallpaper.hwnd().map_err(native_window_error)?.0;

        // SAFETY: HWND Tauri'nin canlı wallpaper penceresinden alınır. ShowWindow
        // child veya top-level olmasına bakmadan aynı native görünürlük bayrağını
        // kapatır; ardından sonuç doğrudan Win32'den doğrulanır.
        unsafe {
            ShowWindow(window, SW_HIDE);
            if IsWindowVisible(window) != 0 {
                Err("Wallpaper native olarak gizlenemedi.".to_string())
            } else {
                Ok(())
            }
        }
    }

    pub fn attach(app: &tauri::AppHandle, monitor_id: Option<&str>) -> Result<Attachment, String> {
        let wallpaper = app
            .get_webview_window("wallpaper")
            .ok_or_else(|| "Wallpaper penceresi bulunamadı.".to_string())?;
        let window = wallpaper.hwnd().map_err(native_window_error)?.0;
        let worker = find_workerw()?;
        let bounds = crate::monitors::selected_bounds(app, monitor_id)?;

        // SAFETY: `window` Tauri'nin canlı wallpaper HWND'si, `worker` ise bu
        // çağrıda EnumWindows ile bulunan canlı WorkerW HWND'sidir. Tüm Win32
        // dönüş değerleri kontrol edilir ve ara hata halinde eski durum yüklenir.
        unsafe {
            let original_parent = GetParent(window);
            let original_style = GetWindowLongPtrW(window, GWL_STYLE);
            let child_style = (original_style & !(WS_POPUP as isize)) | WS_CHILD as isize;
            set_window_style(window, child_style)?;

            if let Err(error) = set_parent_checked(window, worker) {
                let _ = set_window_style(window, original_style);
                return Err(error);
            }

            let mut worker_rect = RECT::default();
            if GetWindowRect(worker, &mut worker_rect) == 0 {
                let error = last_error("WorkerW sınırları okunamadı");
                rollback(window, original_parent, original_style);
                return Err(error);
            }

            // Child window koordinatları parent'ın sol üst köşesine göredir.
            let x = bounds.x - worker_rect.left;
            let y = bounds.y - worker_rect.top;
            if SetWindowPos(
                window,
                ptr::null_mut(),
                x,
                y,
                bounds.width as i32,
                bounds.height as i32,
                SWP_FRAMECHANGED | SWP_NOACTIVATE,
            ) == 0
            {
                let error = last_error("Wallpaper WorkerW içinde boyutlandırılamadı");
                rollback(window, original_parent, original_style);
                return Err(error);
            }

            Ok(Attachment {
                window: window as isize,
                worker: worker as isize,
                original_parent: original_parent as isize,
                original_style,
            })
        }
    }

    pub fn attachment_is_valid(attachment: Attachment) -> bool {
        let window = attachment.window as HWND;
        let worker = attachment.worker as HWND;
        // SAFETY: Handle'lar yalnızca kimlik doğrulaması için kullanılır. IsWindow
        // her iki değeri de kontrol ettikten sonra mevcut parent karşılaştırılır.
        unsafe { IsWindow(window) != 0 && IsWindow(worker) != 0 && GetParent(window) == worker }
    }

    pub fn detach(app: &tauri::AppHandle, attachment: Attachment) -> Result<(), String> {
        let window = attachment.window as HWND;
        let parent = attachment.original_parent as HWND;

        // SAFETY: Handle ve stiller aynı süreçte başarılı `attach` çağrısından
        // kaydedildi. Pencere hâlâ mevcut değilse Win32 hatası kullanıcıya döner.
        unsafe {
            set_parent_checked(window, parent)?;
            set_window_style(window, attachment.original_style)?;
            if SetWindowPos(
                window,
                ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            ) == 0
            {
                return Err(last_error("Wallpaper pencere stili yenilenemedi"));
            }
        }

        // Parent kaldırıldıktan sonra koordinatlar yeniden ekran koordinatıdır.
        // Tauri tarafı bir sonraki gösterimde monitör yerleşimini tekrar uygular.
        let _ = app;
        Ok(())
    }

    fn find_workerw() -> Result<HWND, String> {
        let progman_class = wide("Progman");
        let shell_view_class = wide("SHELLDLL_DefView");
        let worker_class = wide("WorkerW");

        // SAFETY: UTF-16 dizileri fonksiyon süresince yaşar ve null ile biter.
        // Enum callback'e verilen pointer aynı stack frame içindeki HWND alanıdır.
        unsafe {
            let progman = FindWindowW(progman_class.as_ptr(), ptr::null());
            if progman.is_null() {
                return Err(last_error("Windows Progman penceresi bulunamadı"));
            }

            let mut message_result = 0usize;
            let _ = SendMessageTimeoutW(
                progman,
                SPAWN_WORKERW_MESSAGE,
                0 as WPARAM,
                0 as LPARAM,
                SMTO_NORMAL,
                1_000,
                &mut message_result,
            );

            let mut search = WorkerSearch {
                shell_view_class: shell_view_class.as_ptr(),
                worker_class: worker_class.as_ptr(),
                found: ptr::null_mut(),
            };
            let _ = EnumWindows(
                Some(enum_windows),
                (&mut search as *mut WorkerSearch) as LPARAM,
            );

            if search.found.is_null() {
                Err("Windows WorkerW masaüstü katmanı bulunamadı.".to_string())
            } else {
                Ok(search.found)
            }
        }
    }

    struct WorkerSearch {
        shell_view_class: *const u16,
        worker_class: *const u16,
        found: HWND,
    }

    unsafe extern "system" fn enum_windows(window: HWND, parameter: LPARAM) -> BOOL {
        // SAFETY: `parameter`, `find_workerw` içindeki canlı WorkerSearch alanını
        // gösterir. Callback yalnızca senkron EnumWindows çağrısı boyunca çalışır.
        let search = unsafe { &mut *(parameter as *mut WorkerSearch) };
        if search.found.is_null() {
            let shell_view = unsafe {
                FindWindowExW(
                    window,
                    ptr::null_mut(),
                    search.shell_view_class,
                    ptr::null(),
                )
            };
            if !shell_view.is_null() {
                search.found = unsafe {
                    FindWindowExW(ptr::null_mut(), window, search.worker_class, ptr::null())
                };
            }
        }
        1
    }

    unsafe fn set_parent_checked(window: HWND, parent: HWND) -> Result<(), String> {
        unsafe { SetLastError(0) };
        let previous = unsafe { SetParent(window, parent) };
        let error = unsafe { GetLastError() };
        // SetParent başarıyla çalıştığında önceki parent NULL olabilir. Bu yüzden
        // yalnızca NULL dönüş + non-zero GetLastError kombinasyonu gerçek hatadır.
        if previous.is_null() && error != 0 {
            Err(format!("Wallpaper parent değiştirilemedi (Win32 {error})."))
        } else {
            Ok(())
        }
    }

    unsafe fn set_window_style(window: HWND, style: isize) -> Result<(), String> {
        unsafe { SetLastError(0) };
        let previous = unsafe { SetWindowLongPtrW(window, GWL_STYLE, style) };
        let error = unsafe { GetLastError() };
        if previous == 0 && error != 0 {
            Err(format!(
                "Wallpaper pencere stili değiştirilemedi (Win32 {error})."
            ))
        } else {
            Ok(())
        }
    }

    unsafe fn rollback(window: HWND, parent: HWND, style: isize) {
        let _ = unsafe { set_parent_checked(window, parent) };
        let _ = unsafe { set_window_style(window, style) };
        let _ = unsafe {
            SetWindowPos(
                window,
                ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            )
        };
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }

    fn last_error(context: &str) -> String {
        // SAFETY: GetLastError yalnızca çağıran thread'in hata kodunu okur.
        let code = unsafe { GetLastError() };
        format!("{context} (Win32 {code}).")
    }

    fn native_window_error(error: tauri::Error) -> String {
        format!("Native wallpaper penceresi alınamadı: {error}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_desired_and_confirmed_wallpaper_visibility_separately() {
        let state = DesktopHostState::default();
        assert!(!state.status(None).visible);
        assert!(!state.wallpaper_is_desired());

        state.request_wallpaper_visibility(true);
        assert!(state.wallpaper_is_desired());
        assert!(!state.status(None).visible);

        state.confirm_wallpaper_visible();
        assert!(state.status(None).visible);

        state.request_wallpaper_visibility(false);
        let closed = state.status(None);
        assert!(!closed.visible);
        assert!(!closed.closing);
        assert_eq!(closed.mode, "window");
        assert!(!state.wallpaper_is_desired());
    }

    #[test]
    fn keeps_lifecycle_state_consistent_across_repeated_control_wallpaper_transitions() {
        let state = DesktopHostState::default();
        for _ in 0..20 {
            state.request_wallpaper_visibility(true);
            assert!(state.wallpaper_is_desired());
            assert!(!state.wallpaper_is_visible());

            state.confirm_wallpaper_visible();
            assert!(state.wallpaper_is_visible());

            state.request_wallpaper_visibility(false);
            assert!(!state.wallpaper_is_desired());
            assert!(!state.wallpaper_is_visible());
            assert_eq!(state.status(None).mode, "window");
        }
    }

    #[test]
    fn exposes_the_native_close_transition_until_registry_cleanup_finishes() {
        let state = DesktopHostState::default();
        state.request_wallpaper_visibility(true);
        state.confirm_wallpaper_visible();

        state.begin_wallpaper_close();
        state.request_wallpaper_visibility(false);
        let closing = state.status(None);
        assert!(!closing.visible);
        assert!(closing.closing);

        state.finish_wallpaper_close();
        assert!(!state.status(None).closing);
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::Attachment;

    pub fn force_hide(_app: &tauri::AppHandle) -> Result<(), String> {
        Ok(())
    }

    pub fn refresh_desktop() -> Result<(), String> {
        Ok(())
    }

    pub fn attachment_is_valid(_attachment: Attachment) -> bool {
        false
    }

    pub fn attach(
        _app: &tauri::AppHandle,
        _monitor_id: Option<&str>,
    ) -> Result<Attachment, String> {
        Err("Masaüstü katmanı şu anda yalnızca Windows'ta destekleniyor.".to_string())
    }

    pub fn detach(_app: &tauri::AppHandle, _attachment: Attachment) -> Result<(), String> {
        Ok(())
    }
}
