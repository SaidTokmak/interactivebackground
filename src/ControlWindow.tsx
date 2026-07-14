import { FormEvent, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { getDesktopHostStatus, hideWallpaper, isTauriRuntime, showWallpaper } from "./taskApi";
import type { DesktopHostStatus } from "./types";
import { useMonitors } from "./useMonitors";
import { useSettings } from "./useSettings";
import { useTasks } from "./useTasks";
import { WallpaperSurface } from "./WallpaperSurface";

export function ControlWindow() {
  const { tasks, error: taskError, addTask, toggleTask, moveTask, removeTask } = useTasks();
  const { settings, settingsError, saveSettings } = useSettings();
  const { monitors, monitorError } = useMonitors();
  const [title, setTitle] = useState("");
  const [time, setTime] = useState("");
  const [opacityDraft, setOpacityDraft] = useState(settings.opacity);
  const [isAdding, setIsAdding] = useState(false);
  const [desktopStatus, setDesktopStatus] = useState<DesktopHostStatus | null>(null);
  const [autoStartEnabled, setAutoStartEnabled] = useState(false);
  const [integrationError, setIntegrationError] = useState("");

  useEffect(() => setOpacityDraft(settings.opacity), [settings.opacity]);
  useEffect(() => {
    if (!isTauriRuntime()) return;
    void isEnabled()
      .then((enabled) => {
        setAutoStartEnabled(enabled);
        setIntegrationError("");
      })
      .catch((reason) => setIntegrationError(String(reason)));
  }, []);
  useEffect(() => {
    void getDesktopHostStatus().then(setDesktopStatus);
    if (!isTauriRuntime()) return;

    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    void listen("desktop-host-changed", () => {
      void getDesktopHostStatus().then(setDesktopStatus);
    }).then((stopListening) => {
      if (disposed) stopListening();
      else unlisten = stopListening;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const completed = tasks.filter((task) => task.status === "done").length;
  const progress = tasks.length === 0 ? 0 : Math.round((completed / tasks.length) * 100);

  async function submitTask(event: FormEvent) {
    event.preventDefault();
    const cleanTitle = title.trim();
    if (!cleanTitle) return;

    const created = await addTask(cleanTitle, time || null);
    if (created) {
      setTitle("");
      setTime("");
      setIsAdding(false);
    }
  }

  async function openWallpaper() {
    const status = await showWallpaper();
    setDesktopStatus(status);
  }

  async function closeWallpaper() {
    await hideWallpaper();
    setDesktopStatus({ attached: false, mode: "window", warning: null });
  }

  async function toggleWallpaper() {
    if (desktopStatus && desktopStatus.mode !== "window") await closeWallpaper();
    else await openWallpaper();
  }

  async function updateAutoStart(enabled: boolean) {
    setAutoStartEnabled(enabled);
    try {
      if (enabled) await enable();
      else await disable();
      setAutoStartEnabled(await isEnabled());
      setIntegrationError("");
    } catch (reason) {
      setAutoStartEnabled(!enabled);
      setIntegrationError(String(reason));
    }
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <div className="brand-lockup">
          <span className="brand-mark" aria-hidden="true">✦</span>
          <div><strong>interactivebackground</strong><span>Masaüstü çalışma alanın</span></div>
        </div>
        <div className="header-actions">
          <span className="status-dot"><i /> SQLite bağlı</span>
          <button className="header-button" onClick={() => void toggleWallpaper()}>
            {desktopStatus && desktopStatus.mode !== "window" ? "Masaüstünü kapat" : "Masaüstünü aç ↗"}
          </button>
          <button className="icon-button" aria-label="Ayarları aç">⚙</button>
        </div>
      </header>

      <div className="workspace">
        <aside className="control-panel">
          <div className="panel-heading">
            <div><p className="eyebrow">BUGÜN</p><h1>Akışını düzenle</h1></div>
            <span className="date-badge">14 Temmuz</span>
          </div>

          <div className="progress-block">
            <div className="progress-copy"><strong>{completed}/{tasks.length} tamamlandı</strong><span>%{progress} günlük ilerleme</span></div>
            <div className="progress-track"><span style={{ width: `${progress}%` }} /></div>
          </div>

          <section className="manager-section">
            <div className="section-title">
              <h2>Görevler</h2>
              <button className="text-button" onClick={() => setIsAdding((value) => !value)}>{isAdding ? "Vazgeç" : "+ Yeni görev"}</button>
            </div>

            {isAdding && (
              <form className="task-form" onSubmit={submitTask}>
                <label><span>Görev başlığı</span><input autoFocus maxLength={120} value={title} onChange={(event) => setTitle(event.target.value)} placeholder="Ne yapmak istiyorsun?" /></label>
                <label className="time-field"><span>Saat</span><input type="time" value={time} onChange={(event) => setTime(event.target.value)} /></label>
                <button className="primary-button" type="submit">Görevi ekle</button>
              </form>
            )}

            {(taskError || settingsError || monitorError || integrationError || desktopStatus?.warning) && <p className="error-message" role="alert">{taskError || settingsError || monitorError || integrationError || desktopStatus?.warning}</p>}

            <div className="manager-list">
              {tasks.map((task) => (
                <article className={`manager-task ${task.status === "done" ? "is-done" : ""}`} key={task.id}>
                  <button className="check-button" onClick={() => void toggleTask(task.id)} aria-label={`${task.title} görevini tamamla`}>{task.status === "done" ? "✓" : ""}</button>
                  <div className="manager-task-copy"><strong>{task.title}</strong><span>{statusLabel(task.status)}</span></div>
                  <time>{task.scheduledFor ?? "—"}</time>
                  <button className="delete-button" onClick={() => void removeTask(task.id)} aria-label={`${task.title} görevini sil`}>×</button>
                </article>
              ))}
            </div>
          </section>
        </aside>

        <section className="preview-area">
          <div className="preview-toolbar">
            <div><p className="eyebrow">CANLI ÖNİZLEME</p><h2>Masaüstün</h2></div>
            <div className="view-switch" aria-label="Widget şablonu">
              <button className={settings.template === "focus" ? "active" : ""} onClick={() => void saveSettings({ ...settings, template: "focus" })}>Odak</button>
              <button className={settings.template === "kanban" ? "active" : ""} onClick={() => void saveSettings({ ...settings, template: "kanban" })}>Kanban</button>
            </div>
          </div>

          <WallpaperSurface tasks={tasks} template={settings.template} editMode={settings.editMode} opacity={opacityDraft} onToggle={(id) => void toggleTask(id)} onMove={(id, status) => void moveTask(id, status)} />

          <div className="preview-controls">
            <label className="switch-row">
              <input type="checkbox" checked={settings.editMode} onChange={(event) => void saveSettings({ ...settings, editMode: event.target.checked })} />
              <span><b>Düzenleme modu</b><small>{settings.editMode ? "Tıklanabilir overlay açık" : "İkonların arkasında sakin görünüm"}</small></span>
            </label>
            <label className="switch-row autostart-control">
              <input
                type="checkbox"
                checked={autoStartEnabled}
                onChange={(event) => void updateAutoStart(event.target.checked)}
              />
              <span><b>Windows ile başlat</b><small>Açılışta sistem tepsisinde sessizce çalışır</small></span>
            </label>
            <label className="opacity-control">
              <span>Saydamlık <b>%{opacityDraft}</b></span>
              <input
                type="range"
                min="58"
                max="96"
                value={opacityDraft}
                onChange={(event) => setOpacityDraft(Number(event.target.value))}
                onPointerUp={() => void saveSettings({ ...settings, opacity: opacityDraft })}
                onKeyUp={() => void saveSettings({ ...settings, opacity: opacityDraft })}
              />
            </label>
            <label className="monitor-control">
              <span>Hedef ekran</span>
              <select value={selectedMonitorId(settings.monitorId, monitors)} onChange={(event) => void saveSettings({ ...settings, monitorId: event.target.value })}>
                {monitors.map((monitor) => (
                  <option value={monitor.id} key={monitor.id}>
                    {monitor.name}{monitor.isPrimary ? " · Ana" : ""} — {monitor.width}×{monitor.height} @{monitor.scaleFactor.toFixed(2)}x
                  </option>
                ))}
              </select>
            </label>
            <label className="monitor-control auto-calm-control">
              <span>Otomatik sakin mod</span>
              <select
                value={settings.autoCalmMinutes ?? 0}
                onChange={(event) => void saveSettings({
                  ...settings,
                  autoCalmMinutes: Number(event.target.value) || null,
                })}
              >
                <option value={0}>Kapalı</option>
                <option value={1}>1 dakika</option>
                <option value={5}>5 dakika</option>
                <option value={10}>10 dakika</option>
                <option value={15}>15 dakika</option>
              </select>
            </label>
          </div>
        </section>
      </div>
    </main>
  );
}

function selectedMonitorId(currentId: string | null, monitors: import("./types").MonitorInfo[]) {
  if (currentId && monitors.some((monitor) => monitor.id === currentId)) return currentId;
  return monitors.find((monitor) => monitor.isPrimary)?.id ?? monitors[0]?.id ?? "";
}

function statusLabel(status: "todo" | "inProgress" | "done") {
  if (status === "inProgress") return "Devam ediyor";
  if (status === "done") return "Tamamlandı";
  return "Yapılacak";
}
