import { useEffect } from "react";
import { hideWallpaper, isTauriRuntime, recordInteractionActivity } from "./taskApi";
import { useSettings } from "./useSettings";
import { useTasks } from "./useTasks";
import { useTheme } from "./useTheme";
import { WallpaperSurface } from "./WallpaperSurface";

export function WallpaperWindow() {
  const { tasks, error, toggleTask, moveTask } = useTasks();
  const { settings, settingsError, saveSettings } = useSettings();

  useTheme(settings.theme);

  useEffect(() => {
    if (!settings.editMode || !isTauriRuntime()) return;

    let lastReported = 0;
    const reportActivity = () => {
      const now = Date.now();
      if (now - lastReported < 15_000) return;
      lastReported = now;
      void recordInteractionActivity();
    };

    reportActivity();
    window.addEventListener("pointermove", reportActivity);
    window.addEventListener("pointerdown", reportActivity);
    window.addEventListener("keydown", reportActivity);
    return () => {
      window.removeEventListener("pointermove", reportActivity);
      window.removeEventListener("pointerdown", reportActivity);
      window.removeEventListener("keydown", reportActivity);
    };
  }, [settings.editMode]);

  return (
    <main className="wallpaper-window">
      {settings.editMode && <div className="wallpaper-window-controls">
        <div className="view-switch" aria-label="Wallpaper şablonu">
          <button className={settings.template === "focus" ? "active" : ""} onClick={() => void saveSettings({ ...settings, template: "focus" })}>Odak</button>
          <button className={settings.template === "kanban" ? "active" : ""} onClick={() => void saveSettings({ ...settings, template: "kanban" })}>Kanban</button>
        </div>
        <label className="wallpaper-edit-toggle"><input type="checkbox" checked={settings.editMode} onChange={(event) => void saveSettings({ ...settings, editMode: event.target.checked })} /> Etkileşim</label>
        <button className="wallpaper-close" onClick={() => void hideWallpaper()}>Yönetim paneline dön</button>
      </div>}

      {(error || settingsError) && <p className="wallpaper-error" role="alert">{error || settingsError}</p>}
      <WallpaperSurface actual tasks={tasks} template={settings.template} editMode={settings.editMode} opacity={settings.opacity} onToggle={(id) => void toggleTask(id)} onMove={(id, status) => void moveTask(id, status)} />
    </main>
  );
}
