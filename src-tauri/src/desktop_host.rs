use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopHostStatus {
    pub attached: bool,
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
}

impl DesktopHostState {
    pub fn destroy_wallpaper_window(&self, app: &AppHandle) -> Result<(), String> {
        // Pencere WorkerW'nin child'ı olsa bile HWND'yi doğrudan yok etmek,
        // Explorer/DWM'nin ikinci monitörde eski top-level çerçeveyi yeniden
        // çizmesini engeller. Bir sonraki açılışta pencere yeniden oluşturulur.
        *self.lock()? = None;
        self.interaction_mode.store(false, Ordering::Release);
        self.fallback_mode.store(false, Ordering::Release);
        self.workerw_desired.store(false, Ordering::Release);

        if let Some(wallpaper) = app.get_webview_window("wallpaper") {
            wallpaper.destroy().map_err(window_error)?;
        }
        platform::refresh_desktop()?;
        Ok(())
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
        eprintln!("Flowdesk wallpaper WorkerW masaüstü katmanına bağlandı.");
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
        eprintln!("Flowdesk wallpaper WorkerW masaüstü katmanından ayrıldı.");
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
        eprintln!("Flowdesk tıklanabilir etkileşim katmanına geçti.");
        Ok(())
    }

    pub fn leave_interaction_mode(&self, app: &AppHandle) -> Result<(), String> {
        let wallpaper = wallpaper_window(app)?;
        wallpaper.set_always_on_top(false).map_err(window_error)?;
        self.interaction_mode.store(false, Ordering::Release);
        Ok(())
    }

    pub fn is_interaction_mode(&self) -> bool {
        self.interaction_mode.load(Ordering::Acquire)
    }

    pub fn set_fallback_mode(&self, active: bool) {
        self.fallback_mode.store(active, Ordering::Release);
    }

    pub fn status(&self, warning: Option<String>) -> DesktopHostStatus {
        let attached = self.is_attached();
        let interaction = self.is_interaction_mode();
        let fallback = self.fallback_mode.load(Ordering::Acquire);
        DesktopHostStatus {
            attached,
            mode: if attached {
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
