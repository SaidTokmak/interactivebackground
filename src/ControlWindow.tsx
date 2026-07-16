import { FormEvent, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { chooseBackgroundImage as openBackgroundImage, completeOnboarding, getDesktopHostStatus, getOnboardingStatus, hideWallpaper, isTauriRuntime, showWallpaper } from "./taskApi";
import type { BackgroundFit, BackgroundPreset, DesktopHostStatus, LanguagePreference, OnboardingPreferences, ThemePreference, WidgetKind } from "./types";
import { useI18n } from "./i18n";
import type { TranslationKey } from "./i18n/locales/en";
import { useBackgroundSettings } from "./useBackgroundSettings";
import { useMonitors } from "./useMonitors";
import { useSettings } from "./useSettings";
import { useTasks } from "./useTasks";
import { useTheme } from "./useTheme";
import { useDesktopWidgets } from "./useDesktopWidgets";
import { WallpaperSurface } from "./WallpaperSurface";
import appIcon from "./assets/interactivebackground-icon.png";
import { OnboardingWizard } from "./OnboardingWizard";
import { UpdateControl } from "./UpdateControl";
import { useLayoutGrid } from "./useLayoutGrid";
import { MonitorPreview } from "./MonitorPreview";
import { monitorLayoutViewport } from "./widgetLayout";

export function ControlWindow() {
  const { tasks, error: taskError, addTask, toggleTask, moveTask, removeTask } = useTasks();
  const { settings, settingsError, saveSettings, refreshSettings } = useSettings();
  const { monitors, monitorError } = useMonitors();
  const { background, backgroundError, saveBackground } = useBackgroundSettings(settings.monitorId);
  const { widgets, pomodoros, widgetError, addWidget, saveWidget, duplicateWidget, removeWidget, moveWidget, controlPomodoro, savePomodoroDurations } = useDesktopWidgets(settings.monitorId, settings.language);
  const [title, setTitle] = useState("");
  const [time, setTime] = useState("");
  const [opacityDraft, setOpacityDraft] = useState(settings.opacity);
  const [isAdding, setIsAdding] = useState(false);
  const [desktopStatus, setDesktopStatus] = useState<DesktopHostStatus | null>(null);
  const [autoStartEnabled, setAutoStartEnabled] = useState(false);
  const [integrationError, setIntegrationError] = useState("");
  const [overlayDraft, setOverlayDraft] = useState(background.overlay);
  const [blurDraft, setBlurDraft] = useState(background.blur);
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [onboardingCompleted, setOnboardingCompleted] = useState(false);
  const [selectedWidgetId, setSelectedWidgetId] = useState<number | null>(null);
  const [widgetPickerOpen, setWidgetPickerOpen] = useState(false);
  const [inspectorTab, setInspectorTab] = useState<"widget" | "background" | "workspace">("widget");
  const { gridSize, setGridSize, gridSizes } = useLayoutGrid();

  useTheme(settings.theme);
  const { t, formatDate, localizeError } = useI18n(settings.language);

  useEffect(() => setOpacityDraft(settings.opacity), [settings.opacity]);
  useEffect(() => setOverlayDraft(background.overlay), [background.overlay]);
  useEffect(() => setBlurDraft(background.blur), [background.blur]);
  useEffect(() => {
    if (widgets.length === 0) setSelectedWidgetId(null);
    else if (!widgets.some((widget) => widget.id === selectedWidgetId)) setSelectedWidgetId(widgets[0].id);
  }, [selectedWidgetId, widgets]);
  useEffect(() => {
    void getOnboardingStatus()
      .then((status) => {
        setOnboardingCompleted(status.completed);
        setOnboardingOpen(!status.completed);
      })
      .catch((reason) => setIntegrationError(String(reason)));
  }, []);
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
  const selectedMonitor = monitors.find((monitor) => monitor.id === settings.monitorId)
    ?? monitors.find((monitor) => monitor.isPrimary)
    ?? monitors[0];
  const layoutViewport = selectedMonitor ? monitorLayoutViewport(selectedMonitor) : undefined;

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
    setDesktopStatus({ attached: false, visible: false, mode: "window", warning: null });
  }

  async function toggleWallpaper() {
    if (desktopStatus?.visible) await closeWallpaper();
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

  async function finishOnboarding(preferences: OnboardingPreferences, startWithWindows: boolean) {
    await completeOnboarding(preferences);
    setOnboardingCompleted(true);
    setOnboardingOpen(false);
    await refreshSettings();
    if (!isTauriRuntime()) {
      setAutoStartEnabled(startWithWindows);
      return;
    }
    try {
      if (startWithWindows) await enable();
      else await disable();
      setAutoStartEnabled(await isEnabled());
      setIntegrationError("");
    } catch (reason) {
      setIntegrationError(String(reason));
    }
  }

  async function chooseBackgroundImage() {
    try {
      const managedPath = await openBackgroundImage(t("background.imageFilter"));
      if (!managedPath) return;
      await saveBackground({ ...background, source: "custom", customPath: managedPath });
      setIntegrationError("");
    } catch (reason) {
      setIntegrationError(String(reason));
    }
  }

  function selectPreset(preset: BackgroundPreset) {
    void saveBackground({ ...background, source: "preset", preset, customPath: null });
  }

  async function createWidget(kind: WidgetKind) {
    const widget = await addWidget(kind);
    if (widget) setSelectedWidgetId(widget.id);
    setWidgetPickerOpen(false);
    setInspectorTab("widget");
  }

  const selectedWidget = widgets.find((widget) => widget.id === selectedWidgetId) ?? null;
  const selectedWidgetIndex = selectedWidget ? widgets.findIndex((widget) => widget.id === selectedWidget.id) : -1;

  return (
    <main className="app-shell">
      <header className="app-header">
        <div className="brand-lockup">
          <img className="brand-mark" src={appIcon} alt="" aria-hidden="true" />
          <div><strong>interactivebackground</strong><span>{t("brand.subtitle")}</span></div>
        </div>
        <div className="header-actions">
          <span className="status-dot"><i /> {t("status.sqliteConnected")}</span>
          <UpdateControl t={t} />
          <button className="header-button" onClick={() => void toggleWallpaper()}>
            {desktopStatus?.visible ? t("header.closeDesktop") : t("header.openDesktop")}
          </button>
          <button className="icon-button" onClick={() => setOnboardingOpen(true)} aria-label={t("onboarding.reopen")}>⚙</button>
        </div>
      </header>

      <div className="workspace management-workspace">
        <aside className="control-panel">
          <div className="panel-heading">
            <div><p className="eyebrow">{t("dashboard.today")}</p><h1>{t("dashboard.organize")}</h1></div>
            <span className="date-badge">{formatDate(new Date())}</span>
          </div>

          <div className="progress-block">
            <div className="progress-copy"><strong>{t("progress.completed", { completed, total: tasks.length })}</strong><span>{t("progress.daily", { progress })}</span></div>
            <div className="progress-track"><span style={{ width: `${progress}%` }} /></div>
          </div>

          <section className="manager-section">
            <div className="section-title">
              <h2>{t("tasks.title")}</h2>
              <button className="text-button" onClick={() => setIsAdding((value) => !value)}>{isAdding ? t("tasks.cancel") : t("tasks.new")}</button>
            </div>

            {isAdding && (
              <form className="task-form" onSubmit={submitTask}>
                <label><span>{t("tasks.titleLabel")}</span><input autoFocus maxLength={120} value={title} onChange={(event) => setTitle(event.target.value)} placeholder={t("tasks.titlePlaceholder")} /></label>
                <label className="time-field"><span>{t("tasks.time")}</span><input type="time" value={time} onChange={(event) => setTime(event.target.value)} /></label>
                <button className="primary-button" type="submit">{t("tasks.add")}</button>
              </form>
            )}

            {(taskError || settingsError || backgroundError || widgetError || monitorError || integrationError || desktopStatus?.warning) && <p className="error-message" role="alert">{localizeError(taskError || settingsError || backgroundError || widgetError || monitorError || integrationError || desktopStatus?.warning || "")}</p>}

            <div className="manager-list">
              {tasks.map((task) => (
                <article className={`manager-task ${task.status === "done" ? "is-done" : ""}`} key={task.id}>
                  <button className="check-button" onClick={() => void toggleTask(task.id)} aria-label={t("tasks.toggleAria", { title: task.title })}>{task.status === "done" ? "✓" : ""}</button>
                  <div className="manager-task-copy"><strong>{task.title}</strong><span>{statusLabel(task.status, t)}</span></div>
                  <time>{task.scheduledFor ?? "—"}</time>
                  <button className="delete-button" onClick={() => void removeTask(task.id)} aria-label={t("tasks.deleteAria", { title: task.title })}>×</button>
                </article>
              ))}
            </div>
          </section>
        </aside>

        <section className="preview-area canvas-panel">
          <div className="preview-toolbar">
            <div><p className="eyebrow">{t("preview.live")}</p><h2>{t("preview.desktop")}</h2></div>
            <div className="preview-tools">
              <label>{t("layout.gridDensity")}
                <select value={gridSize} onChange={(event) => setGridSize(Number(event.target.value))}>
                  {gridSizes.map((size) => <option value={size} key={size}>{t(size === 0.005 ? "layout.gridFine" : "layout.gridNormal")}</option>)}
                </select>
              </label>
              <span className="widget-count">{t("widgets.count", { count: widgets.length })}</span>
            </div>
          </div>

          <MonitorPreview viewport={layoutViewport}>
            <WallpaperSurface tasks={tasks} widgets={widgets} pomodoros={pomodoros} editMode={settings.editMode} opacity={opacityDraft} language={settings.language} background={{ ...background, overlay: overlayDraft, blur: blurDraft }} layoutViewport={layoutViewport} gridSize={gridSize} onToggle={(id) => void toggleTask(id)} onMove={(id, status) => void moveTask(id, status)} onWidgetChange={(widget) => void saveWidget(widget)} onPomodoroAction={(id, action) => void controlPomodoro(id, action)} />
          </MonitorPreview>
        </section>

        <aside className="inspector-panel">
          <div className="inspector-heading">
            <div><p className="eyebrow">{t("inspector.eyebrow")}</p><h2>{t("inspector.title")}</h2></div>
            <div className="add-widget-wrap">
              <button className="add-widget-button" onClick={() => setWidgetPickerOpen((open) => !open)}>＋ {t("widgets.add")}</button>
              {widgetPickerOpen && <div className="widget-picker" role="dialog" aria-label={t("widgets.catalogAria")}>
                {(["focus", "kanban", "pomodoro", "clock", "date", "dailyPoem", "dailyVerse", "dailyHadith"] as WidgetKind[]).map((kind) => (
                  <button onClick={() => void createWidget(kind)} key={kind}><span className={`widget-kind-icon kind-${kind}`}>{widgetIcon(kind)}</span><b>{widgetKindLabel(kind, t)}</b></button>
                ))}
              </div>}
            </div>
          </div>

          <div className="inspector-tabs" role="tablist">
            {(["widget", "background", "workspace"] as const).map((tab) => <button role="tab" aria-selected={inspectorTab === tab} className={inspectorTab === tab ? "active" : ""} onClick={() => setInspectorTab(tab)} key={tab}>{t(`inspector.${tab}` as TranslationKey)}</button>)}
          </div>

          <div className="inspector-scroll">
            {inspectorTab === "widget" && <>
              <div className="widget-selector" aria-label={t("widgets.selectAria")}>
                {widgets.map((widget) => <button className={widget.id === selectedWidgetId ? "selected" : ""} onClick={() => setSelectedWidgetId(widget.id)} key={widget.id}><span className={`widget-kind-icon kind-${widget.kind}`}>{widgetIcon(widget.kind)}</span><span><b>{widgetKindLabel(widget.kind, t)}</b><small>{widget.visible ? t("widgets.visible") : t("widgets.hidden")}</small></span></button>)}
              </div>
              {!selectedWidget && <p className="widget-empty">{t("widgets.empty")}</p>}
              {selectedWidget && <section className="selected-widget-card">
                <div className="selected-widget-title"><span className={`widget-kind-icon kind-${selectedWidget.kind}`}>{widgetIcon(selectedWidget.kind)}</span><div><h3>{widgetKindLabel(selectedWidget.kind, t)}</h3><span>#{selectedWidget.id}</span></div></div>
                <div className="inspector-switches">
                  <label className="switch-row"><input type="checkbox" checked={selectedWidget.visible} onChange={(event) => void saveWidget({ ...selectedWidget, visible: event.target.checked })} /><span><b>{t("widgets.visible")}</b><small>{t("widgets.visibilityHelp")}</small></span></label>
                  <label className="switch-row"><input type="checkbox" checked={selectedWidget.locked} onChange={(event) => void saveWidget({ ...selectedWidget, locked: event.target.checked })} /><span><b>{t("layout.lock")}</b><small>{t("widgets.lockHelp")}</small></span></label>
                  <label className="switch-row"><input type="checkbox" checked={selectedWidget.snapToGrid} onChange={(event) => void saveWidget({ ...selectedWidget, snapToGrid: event.target.checked })} /><span><b>{t("layout.grid")}</b><small>{t("widgets.gridHelp")}</small></span></label>
                </div>
                {selectedWidget.kind === "pomodoro" && pomodoros[selectedWidget.id] && <div className="pomodoro-settings inspector-pomodoro">
                  <label>{t("pomodoro.workShort")}<input key={`work-${pomodoros[selectedWidget.id].workMinutes}`} type="number" min="1" max="180" defaultValue={pomodoros[selectedWidget.id].workMinutes} onBlur={(event) => void savePomodoroDurations(selectedWidget.id, Number(event.currentTarget.value), pomodoros[selectedWidget.id].breakMinutes)} /></label>
                  <label>{t("pomodoro.breakShort")}<input key={`break-${pomodoros[selectedWidget.id].breakMinutes}`} type="number" min="1" max="60" defaultValue={pomodoros[selectedWidget.id].breakMinutes} onBlur={(event) => void savePomodoroDurations(selectedWidget.id, pomodoros[selectedWidget.id].workMinutes, Number(event.currentTarget.value))} /></label>
                </div>}
                <div className="inspector-actions">
                  <button disabled={selectedWidgetIndex === 0} onClick={() => void moveWidget(selectedWidget.id, -1)}>↑ {t("widgets.moveUpShort")}</button>
                  <button disabled={selectedWidgetIndex === widgets.length - 1} onClick={() => void moveWidget(selectedWidget.id, 1)}>↓ {t("widgets.moveDownShort")}</button>
                  <button onClick={() => void duplicateWidget(selectedWidget.id)}>{t("widgets.duplicate")}</button>
                  <button className="danger" onClick={() => void removeWidget(selectedWidget.id)}>{t("widgets.remove")}</button>
                </div>
              </section>}
            </>}

            {inspectorTab === "background" && <section className="background-panel inspector-section">
              <div className="background-heading"><div><h3>{t("background.title")}</h3><span>{t("background.subtitle")}</span></div><button className="background-file-button" onClick={() => void chooseBackgroundImage()}>{background.source === "custom" ? t("background.replace") : t("background.choose")}</button></div>
              <div className="background-options">
                {(["foldedHorizon", "midnight", "graphite", "ember"] as BackgroundPreset[]).map((preset) => { const name = t(`background.${preset}` as TranslationKey); return <button className={`background-option ${background.source === "preset" && background.preset === preset ? "active" : ""}`} aria-label={t("background.presetAria", { name })} onClick={() => selectPreset(preset)} key={preset}><span className={`background-swatch preset-${preset}`} /><b>{name}</b></button>; })}
                <button className={`background-option custom-option ${background.source === "custom" ? "active" : ""}`} onClick={() => void chooseBackgroundImage()}><span className="background-swatch custom-swatch">＋</span><b>{t("background.custom")}</b></button>
              </div>
              <div className="background-adjustments">
                <label className="monitor-control background-fit-control"><span>{t("background.fit")}</span><select disabled={background.source !== "custom"} value={background.fit} onChange={(event) => void saveBackground({ ...background, fit: event.target.value as BackgroundFit })}><option value="cover">{t("background.fitCover")}</option><option value="contain">{t("background.fitContain")}</option><option value="stretch">{t("background.fitStretch")}</option></select></label>
                <label className="opacity-control background-range"><span>{t("background.overlay")} <b>%{overlayDraft}</b></span><input type="range" min="0" max="70" value={overlayDraft} onChange={(event) => setOverlayDraft(Number(event.target.value))} onPointerUp={() => void saveBackground({ ...background, overlay: overlayDraft })} onKeyUp={() => void saveBackground({ ...background, overlay: overlayDraft })} /></label>
                <label className="opacity-control background-range"><span>{t("background.blur")} <b>{blurDraft}px</b></span><input type="range" min="0" max="24" value={blurDraft} onChange={(event) => setBlurDraft(Number(event.target.value))} onPointerUp={() => void saveBackground({ ...background, blur: blurDraft })} onKeyUp={() => void saveBackground({ ...background, blur: blurDraft })} /></label>
              </div>
            </section>}

            {inspectorTab === "workspace" && <div className="preview-controls inspector-section">
            <label className="monitor-control theme-control">
              <span>{t("theme.label")}</span>
              <select
                value={settings.theme}
                onChange={(event) => void saveSettings({
                  ...settings,
                  theme: event.target.value as ThemePreference,
                })}
              >
                <option value="system">{t("theme.system")}</option>
                <option value="light">{t("theme.light")}</option>
                <option value="dark">{t("theme.dark")}</option>
              </select>
            </label>
            <label className="monitor-control language-control">
              <span>{t("language.label")}</span>
              <select
                value={settings.language}
                onChange={(event) => void saveSettings({
                  ...settings,
                  language: event.target.value as LanguagePreference,
                })}
              >
                <option value="system">{t("language.system")}</option>
                <option value="tr">{t("language.tr")}</option>
                <option value="en">{t("language.en")}</option>
              </select>
            </label>
            <label className="switch-row">
              <input type="checkbox" checked={settings.editMode} onChange={(event) => void saveSettings({ ...settings, editMode: event.target.checked })} />
              <span><b>{t("edit.label")}</b><small>{settings.editMode ? t("edit.activeHelp") : t("edit.calmHelp")}</small></span>
            </label>
            <label className="switch-row autostart-control">
              <input
                type="checkbox"
                checked={autoStartEnabled}
                onChange={(event) => void updateAutoStart(event.target.checked)}
              />
              <span><b>{t("autostart.label")}</b><small>{t("autostart.help")}</small></span>
            </label>
            <label className="opacity-control">
              <span>{t("opacity.label")} <b>%{opacityDraft}</b></span>
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
              <span>{t("monitor.label")}</span>
              <select value={selectedMonitorId(settings.monitorId, monitors)} onChange={(event) => void saveSettings({ ...settings, monitorId: event.target.value })}>
                {monitors.map((monitor) => (
                  <option value={monitor.id} key={monitor.id}>
                    {monitor.id.startsWith("browser:") ? t("monitor.browserDisplay") : monitor.name}{monitor.isPrimary ? ` · ${t("monitor.primary")}` : ""} — {monitor.width}×{monitor.height} @{monitor.scaleFactor.toFixed(2)}x
                  </option>
                ))}
              </select>
            </label>
            <label className="monitor-control auto-calm-control">
              <span>{t("autoCalm.label")}</span>
              <select
                value={settings.autoCalmMinutes ?? 0}
                onChange={(event) => void saveSettings({
                  ...settings,
                  autoCalmMinutes: Number(event.target.value) || null,
                })}
              >
                <option value={0}>{t("autoCalm.off")}</option>
                <option value={1}>{t("autoCalm.minute")}</option>
                <option value={5}>{t("autoCalm.minutes", { count: 5 })}</option>
                <option value={10}>{t("autoCalm.minutes", { count: 10 })}</option>
                <option value={15}>{t("autoCalm.minutes", { count: 15 })}</option>
              </select>
            </label>
            </div>}
          </div>
        </aside>
      </div>
      {onboardingOpen && <OnboardingWizard
        settings={settings}
        monitors={monitors}
        autoStartEnabled={autoStartEnabled}
        initialBackgroundPreset={background.preset}
        canDismiss={onboardingCompleted}
        onDismiss={() => setOnboardingOpen(false)}
        onComplete={finishOnboarding}
      />}
    </main>
  );
}

function selectedMonitorId(currentId: string | null, monitors: import("./types").MonitorInfo[]) {
  if (currentId && monitors.some((monitor) => monitor.id === currentId)) return currentId;
  return monitors.find((monitor) => monitor.isPrimary)?.id ?? monitors[0]?.id ?? "";
}

function statusLabel(
  status: "todo" | "inProgress" | "done",
  t: (key: "status.todo" | "status.inProgress" | "status.done") => string,
) {
  if (status === "inProgress") return t("status.inProgress");
  if (status === "done") return t("status.done");
  return t("status.todo");
}

function widgetKindLabel(kind: WidgetKind, t: (key: TranslationKey) => string) {
  return t(`widgets.kind.${kind}` as TranslationKey);
}

function widgetIcon(kind: WidgetKind) {
  if (kind === "focus") return "✓";
  if (kind === "kanban") return "▦";
  if (kind === "pomodoro") return "◷";
  if (kind === "clock") return "◴";
  if (kind === "date") return "▣";
  if (kind === "dailyPoem") return "❧";
  if (kind === "dailyVerse") return "◇";
  return "☾";
}
