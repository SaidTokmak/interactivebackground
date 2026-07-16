import { useEffect } from "react";
import { hideWallpaper, isTauriRuntime, recordInteractionActivity } from "./taskApi";
import { useSettings } from "./useSettings";
import { useTasks } from "./useTasks";
import { useTheme } from "./useTheme";
import { WallpaperSurface } from "./WallpaperSurface";
import { useI18n } from "./i18n";
import { useBackgroundSettings } from "./useBackgroundSettings";
import { useDesktopWidgets } from "./useDesktopWidgets";
import { useMonitors } from "./useMonitors";
import { useLayoutGrid } from "./useLayoutGrid";
import { monitorLayoutViewport } from "./widgetLayout";

export function WallpaperWindow() {
  const { tasks, error, toggleTask, moveTask } = useTasks();
  const { settings, settingsError, saveSettings } = useSettings();
  const { background, backgroundError } = useBackgroundSettings(settings.monitorId);
  const { widgets, pomodoros, widgetError, saveWidget, controlPomodoro } = useDesktopWidgets(settings.monitorId, settings.language);
  const { monitors } = useMonitors();
  const { gridSize, setGridSize, gridSizes } = useLayoutGrid();
  const selectedMonitor = monitors.find((monitor) => monitor.id === settings.monitorId)
    ?? monitors.find((monitor) => monitor.isPrimary)
    ?? monitors[0];
  const layoutViewport = selectedMonitor ? monitorLayoutViewport(selectedMonitor) : undefined;

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
        <span className="wallpaper-widget-count">{t("widgets.count", { count: widgets.length })}</span>
        <label className="wallpaper-edit-toggle"><input type="checkbox" checked={settings.editMode} onChange={(event) => void saveSettings({ ...settings, editMode: event.target.checked })} /> {t("wallpaper.interaction")}</label>
        <label className="wallpaper-grid-density">{t("layout.gridDensity")}<select value={gridSize} onChange={(event) => setGridSize(Number(event.target.value))}>{gridSizes.map((size) => <option value={size} key={size}>{t(size === 0.005 ? "layout.gridFine" : "layout.gridNormal")}</option>)}</select></label>
        <button className="wallpaper-close" onClick={() => void hideWallpaper()}>{t("wallpaper.back")}</button>
      </div>}

      {(error || settingsError || backgroundError || widgetError) && <p className="wallpaper-error" role="alert">{localizeError(error || settingsError || backgroundError || widgetError || "")}</p>}
      <WallpaperSurface actual tasks={tasks} widgets={widgets} pomodoros={pomodoros} editMode={settings.editMode} opacity={settings.opacity} language={settings.language} background={background} layoutViewport={layoutViewport} gridSize={gridSize} onToggle={(id) => void toggleTask(id)} onMove={(id, status) => void moveTask(id, status)} onWidgetChange={(widget) => void saveWidget(widget)} onPomodoroAction={(id, action) => void controlPomodoro(id, action)} />
    </main>
  );
}
