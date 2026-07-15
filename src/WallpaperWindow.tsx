import { useEffect } from "react";
import { hideWallpaper, isTauriRuntime, recordInteractionActivity } from "./taskApi";
import { useSettings } from "./useSettings";
import { useTasks } from "./useTasks";
import { useTheme } from "./useTheme";
import { WallpaperSurface } from "./WallpaperSurface";
import { useI18n } from "./i18n";
import { useBackgroundSettings } from "./useBackgroundSettings";
import { useWidgetLayout } from "./useWidgetLayout";

export function WallpaperWindow() {
  const { tasks, error, toggleTask, moveTask } = useTasks();
  const { settings, settingsError, saveSettings } = useSettings();
  const { background, backgroundError } = useBackgroundSettings(settings.monitorId);
  const { layout, layoutError, saveLayout, restoreLayout } = useWidgetLayout(settings.monitorId, settings.template);

  useTheme(settings.theme);
  const { t, localizeError } = useI18n(settings.language);

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
        <div className="view-switch" aria-label={t("template.wallpaperLabel")}>
          <button className={settings.template === "focus" ? "active" : ""} onClick={() => void saveSettings({ ...settings, template: "focus" })}>{t("template.focus")}</button>
          <button className={settings.template === "kanban" ? "active" : ""} onClick={() => void saveSettings({ ...settings, template: "kanban" })}>{t("template.kanban")}</button>
        </div>
        <button className={`wallpaper-tool ${layout.locked ? "active" : ""}`} onClick={() => void saveLayout({ ...layout, locked: !layout.locked })}>{layout.locked ? t("layout.unlock") : t("layout.lock")}</button>
        <label className="wallpaper-grid-toggle"><input type="checkbox" checked={layout.snapToGrid} onChange={(event) => void saveLayout({ ...layout, snapToGrid: event.target.checked })} /> {t("layout.grid")}</label>
        <button className="wallpaper-tool" onClick={() => void restoreLayout()}>{t("layout.reset")}</button>
        <label className="wallpaper-edit-toggle"><input type="checkbox" checked={settings.editMode} onChange={(event) => void saveSettings({ ...settings, editMode: event.target.checked })} /> {t("wallpaper.interaction")}</label>
        <button className="wallpaper-close" onClick={() => void hideWallpaper()}>{t("wallpaper.back")}</button>
      </div>}

      {(error || settingsError || backgroundError || layoutError) && <p className="wallpaper-error" role="alert">{localizeError(error || settingsError || backgroundError || layoutError || "")}</p>}
      <WallpaperSurface actual tasks={tasks} template={settings.template} editMode={settings.editMode} opacity={settings.opacity} language={settings.language} background={background} layout={layout} onToggle={(id) => void toggleTask(id)} onMove={(id, status) => void moveTask(id, status)} onLayoutChange={(next) => void saveLayout(next)} />
    </main>
  );
}
