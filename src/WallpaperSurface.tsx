import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent as ReactKeyboardEvent, type PointerEvent as ReactPointerEvent, type ReactNode } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { BackgroundSettings, ClockWidgetSettings, DesktopWidget, LanguagePreference, PomodoroAction, PomodoroState, Task, TaskStatus, WidgetKind } from "./types";
import { useI18n } from "./i18n";
import type { TranslationKey } from "./i18n/locales/en";
import { getDailyContent } from "./dailyContent";
import { isTauriRuntime } from "./taskApi";
import { DEFAULT_GRID_SIZE, hasWidgetCollision, SURFACE_MARGIN, widgetSizeLimits, type LayoutViewport } from "./widgetLayout";
import { backgroundPresetTone } from "./backgroundPresets";
import { BackgroundArtwork } from "./BackgroundArtwork";

type Props = {
  tasks: Task[];
  widgets: DesktopWidget[];
  pomodoros: Record<number, PomodoroState>;
  editMode: boolean;
  opacity: number;
  language: LanguagePreference;
  background: BackgroundSettings;
  actual?: boolean;
  layoutViewport?: LayoutViewport;
  gridSize?: number;
  onToggle: (id: number) => void;
  onMove: (id: number, status: TaskStatus) => void;
  onWidgetChange: (widget: DesktopWidget) => void;
  onPomodoroAction: (widgetId: number, action: PomodoroAction) => void;
};

type InteractionMode = "move" | "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

type ActiveInteraction = {
  pointerId: number;
  mode: InteractionMode;
  startX: number;
  startY: number;
  initial: DesktopWidget;
  latest: DesktopWidget;
  lastValid: DesktopWidget;
  valid: boolean;
  bounds: DOMRect;
};

export function WallpaperSurface({ tasks, widgets, pomodoros, editMode, opacity, language, background, actual = false, layoutViewport, gridSize = DEFAULT_GRID_SIZE, onToggle, onMove, onWidgetChange, onPomodoroAction }: Props) {
  const { t, formatDate } = useI18n(language);
  const surfaceRef = useRef<HTMLDivElement>(null);
  const interactionRef = useRef<ActiveInteraction | null>(null);
  const [liveWidgets, setLiveWidgets] = useState(widgets);
  const [invalidWidgetId, setInvalidWidgetId] = useState<number | null>(null);
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    if (!interactionRef.current) setLiveWidgets(widgets);
  }, [widgets]);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1_000);
    return () => window.clearInterval(timer);
  }, []);

  const completed = tasks.filter((task) => task.status === "done").length;
  const progress = tasks.length === 0 ? 0 : Math.round((completed / tasks.length) * 100);
  const nextTask = tasks.find((task) => task.status !== "done");
  const columns = [
    { label: t("kanban.todo"), status: "todo" as const },
    { label: t("kanban.inProgress"), status: "inProgress" as const },
    { label: t("kanban.done"), status: "done" as const },
  ];
  const customImage = background.source === "custom" && background.customPath
    ? (isTauriRuntime() ? convertFileSrc(background.customPath) : background.customPath)
    : null;
  const backgroundStyle = {
    "--background-blur": `${background.blur}px`,
    backgroundImage: customImage ? `url("${customImage}")` : undefined,
    backgroundSize: background.fit === "stretch" ? "100% 100%" : background.fit,
  } as CSSProperties;
  const backgroundTone = background.source === "preset" ? backgroundPresetTone(background.preset) : "dark";

  function beginInteraction(widget: DesktopWidget, mode: InteractionMode, event: ReactPointerEvent<HTMLElement>) {
    if (!editMode || widget.locked || !surfaceRef.current) return;
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    interactionRef.current = {
      pointerId: event.pointerId,
      mode,
      startX: event.clientX,
      startY: event.clientY,
      initial: widget,
      latest: widget,
      lastValid: widget,
      valid: true,
      bounds: surfaceRef.current.getBoundingClientRect(),
    };
  }

  function continueInteraction(event: ReactPointerEvent<HTMLElement>) {
    const active = interactionRef.current;
    if (!active || active.pointerId !== event.pointerId) return;
    const next = calculateLayout(
      active.initial,
      active.mode,
      (event.clientX - active.startX) / active.bounds.width,
      (event.clientY - active.startY) / active.bounds.height,
      layoutViewport ?? active.bounds,
      event.altKey ? 0 : gridSize,
    );
    active.latest = next;
    active.valid = !hasWidgetCollision(next, liveWidgets);
    if (active.valid) active.lastValid = next;
    setInvalidWidgetId(active.valid ? null : next.id);
    setLiveWidgets((current) => current.map((widget) => widget.id === next.id ? next : widget));
  }

  function finishInteraction(event: ReactPointerEvent<HTMLElement>) {
    const active = interactionRef.current;
    if (!active || active.pointerId !== event.pointerId) return;
    interactionRef.current = null;
    setInvalidWidgetId(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    setLiveWidgets((current) => current.map((widget) => widget.id === active.lastValid.id ? active.lastValid : widget));
    if (active.lastValid !== active.initial) onWidgetChange(active.lastValid);
  }

  function nudgeWidget(widget: DesktopWidget, event: ReactKeyboardEvent<HTMLElement>) {
    if (event.target !== event.currentTarget) return;
    if (!editMode || widget.locked || !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) return;
    event.preventDefault();
    event.stopPropagation();
    const step = event.altKey ? 0.001 : gridSize * (event.shiftKey ? 5 : 1);
    const deltaX = event.key === "ArrowLeft" ? -step : event.key === "ArrowRight" ? step : 0;
    const deltaY = event.key === "ArrowUp" ? -step : event.key === "ArrowDown" ? step : 0;
    const viewport = layoutViewport ?? surfaceRef.current?.getBoundingClientRect();
    if (!viewport) return;
    const next = calculateLayout(widget, "move", deltaX, deltaY, viewport, event.altKey ? 0 : gridSize);
    if (hasWidgetCollision(next, liveWidgets)) {
      setInvalidWidgetId(widget.id);
      window.setTimeout(() => setInvalidWidgetId((id) => id === widget.id ? null : id), 180);
      return;
    }
    setLiveWidgets((current) => current.map((item) => item.id === next.id ? next : item));
    onWidgetChange(next);
  }

  function widgetContent(widget: DesktopWidget): ReactNode {
    const controlsDisabled = !editMode;
    const disabledTitle = controlsDisabled ? t("widget.controlsDisabledTitle") : undefined;
    if (widget.kind === "focus") {
      return <>
        <div className="widget-tasks">
          {tasks.slice(0, 6).map((task) => (
            <label className={task.status === "done" ? "done" : ""} key={task.id}>
              <input type="checkbox" checked={task.status === "done"} disabled={controlsDisabled} title={disabledTitle} onChange={() => onToggle(task.id)} />
              <span>{task.title}</span><time>{task.scheduledFor}</time>
            </label>
          ))}
        </div>
        <div className="focus-action"><span>{nextTask ? t("widget.next", { title: nextTask.title }) : t("widget.allDone")}</span></div>
      </>;
    }
    if (widget.kind === "kanban") {
      return <div className="kanban-board">
        {columns.map((column) => (
          <div className="kanban-column" key={column.status}>
            <span>{column.label}</span>
            {tasks.filter((task) => task.status === column.status).map((task) => (
              <button className="kanban-card" disabled={controlsDisabled} title={disabledTitle} key={task.id} onClick={() => onMove(task.id, nextStatus(column.status))}>{task.title}</button>
            ))}
          </div>
        ))}
      </div>;
    }
    if (widget.kind === "pomodoro") {
      const state = pomodoros[widget.id];
      const remaining = pomodoroRemaining(state, now);
      const total = state ? (state.mode === "work" ? state.workMinutes : state.breakMinutes) * 60 : 1_500;
      const percent = Math.max(0, Math.min(100, ((total - remaining) / total) * 100));
      return <div className="pomodoro-body">
        <span className="pomodoro-mode">{state?.mode === "break" ? t("pomodoro.break") : t("pomodoro.work")}</span>
        <strong className="pomodoro-time">{formatDuration(remaining)}</strong>
        <div className="pomodoro-track"><span style={{ width: `${percent}%` }} /></div>
        <div className="pomodoro-actions">
          <button disabled={controlsDisabled} title={disabledTitle} onClick={() => onPomodoroAction(widget.id, state?.running ? "pause" : "start")}>{state?.running ? t("pomodoro.pause") : t("pomodoro.start")}</button>
          <button disabled={controlsDisabled} title={disabledTitle} onClick={() => onPomodoroAction(widget.id, "reset")}>{t("pomodoro.reset")}</button>
          <button disabled={controlsDisabled} title={disabledTitle} onClick={() => onPomodoroAction(widget.id, "skip")}>{t("pomodoro.skip")}</button>
        </div>
      </div>;
    }
    if (widget.kind === "clock") {
      const clock = widget.clockSettings ?? defaultClockSettings();
      const dateLine = formatClockDate(now, language, clock);
      if (clock.style === "analog") {
        const hands = clockHandAngles(now, clock.timeZone);
        return <div className="clock-body analog-clock-body">
          <div className="analog-clock" aria-label={formatClock(now, language, clock)}>
            {Array.from({ length: 12 }, (_, index) => <i className="analog-tick" style={{ transform: `rotate(${index * 30}deg)` }} key={index} />)}
            <span className="analog-hand hour-hand" style={{ transform: `rotate(${hands.hour}deg)` }} />
            <span className="analog-hand minute-hand" style={{ transform: `rotate(${hands.minute}deg)` }} />
            {clock.showSeconds && <span className="analog-hand second-hand" style={{ transform: `rotate(${hands.second}deg)` }} />}
            <b className="analog-pin" />
          </div>
          {dateLine && <span>{dateLine}</span>}
        </div>;
      }
      return <div className="clock-body"><strong>{formatClock(now, language, clock)}</strong>{dateLine && <span>{dateLine}</span>}</div>;
    }
    if (widget.kind === "dailyPoem" || widget.kind === "dailyVerse" || widget.kind === "dailyHadith") {
      const content = getDailyContent(widget.kind, language, now);
      return <div className={`daily-content-body ${widget.kind === "dailyHadith" ? "daily-hadith-body" : ""}`}>
        {content.original && <p className="daily-original" dir="rtl" lang="ar">{content.original}</p>}
        <blockquote className={widget.kind === "dailyHadith" ? "daily-original" : ""} dir={widget.kind === "dailyHadith" ? "rtl" : undefined} lang={widget.kind === "dailyHadith" ? "ar" : undefined}>{content.text}</blockquote>
        {content.note && <p className="daily-note">{content.note}</p>}
        <div className="daily-source-row">
          <span><b>{content.attribution}</b>{content.reference && ` · ${content.reference}`}</span>
          <div className="daily-source-actions">
            {content.originalSourceUrl && <button type="button" disabled={controlsDisabled} title={disabledTitle} onClick={() => void openExternal(content.originalSourceUrl!)}>{t("daily.arabicSource")}</button>}
            <button type="button" disabled={controlsDisabled} title={disabledTitle} onClick={() => void openExternal(content.sourceUrl)}>{t("daily.source")}</button>
          </div>
        </div>
        <small className="daily-license">{content.license}</small>
      </div>;
    }
    return <div className="date-body"><strong>{now.toLocaleDateString(resolveLocale(language), { day: "2-digit" })}</strong><div><span>{now.toLocaleDateString(resolveLocale(language), { month: "long" })}</span><b>{now.toLocaleDateString(resolveLocale(language), { weekday: "long" })}</b></div></div>;
  }

  return (
    <div className={`desktop-preview surface-tone-${backgroundTone} ${actual ? "actual-surface" : ""}`} ref={surfaceRef} style={{
      "--layout-grid-size": `${gridSize * 100}%`,
      "--monitor-aspect": layoutViewport ? `${layoutViewport.width} / ${layoutViewport.height}` : "16 / 9",
      "--monitor-ratio": layoutViewport ? layoutViewport.width / layoutViewport.height : 16 / 9,
    } as CSSProperties}>
      <BackgroundArtwork preset={background.preset} custom={Boolean(customImage)} style={backgroundStyle} />
      <div className="desktop-overlay" style={{ backgroundColor: `rgba(5, 8, 18, ${background.overlay / 100})` }} />
      {editMode && liveWidgets.some((widget) => widget.visible && widget.snapToGrid) && <div className="layout-grid" />}
      {editMode && <div className="desktop-topline"><span className="desktop-mode">⌁ {t("wallpaper.mode.edit")}</span></div>}

      {liveWidgets.filter((widget) => widget.visible).map((widget) => {
        const style = {
          left: `${widget.x * 100}%`, top: `${widget.y * 100}%`, width: `${widget.width * 100}%`, height: `${widget.height * 100}%`,
          zIndex: 3 + widget.sortOrder,
          backgroundColor: `color-mix(in srgb, var(--widget) ${opacity}%, transparent)`,
        } as CSSProperties;
        const taskWidget = widget.kind === "focus" || widget.kind === "kanban";
        const interactiveWidget = widget.kind === "focus" || widget.kind === "kanban" || widget.kind === "pomodoro" || widget.kind === "dailyPoem" || widget.kind === "dailyVerse" || widget.kind === "dailyHadith";
        return <section className={`wallpaper-widget widget-${widget.kind} ${actual ? "actual-widget" : ""} ${editMode ? "editing" : ""} ${widget.locked ? "layout-locked" : ""} ${invalidWidgetId === widget.id ? "layout-invalid" : ""}`} style={style} tabIndex={editMode && !widget.locked ? 0 : undefined} aria-label={editMode && !widget.locked ? `${widgetTitle(widget.kind, t)}. ${t("layout.keyboardHint")}` : undefined} onKeyDown={(event) => nudgeWidget(widget, event)} key={widget.id}>
          {editMode && <div className={`widget-edit-rail ${widget.locked ? "is-locked" : ""}`} title={widget.locked ? t("layout.lockedBadge") : t("layout.keyboardHint")} onPointerDown={(event) => beginInteraction(widget, "move", event)} onPointerMove={continueInteraction} onPointerUp={finishInteraction} onPointerCancel={finishInteraction}>
            <span aria-hidden="true"><i /></span><small>{widget.locked ? `🔒 ${t("layout.lockedBadge")}` : t("layout.dragHere")}</small>
          </div>}
          <div className="widget-header">
            <div><h3>{widgetTitle(widget.kind, t)}</h3>{taskWidget && <span>{formatDate(now, "long")}</span>}{widget.kind === "clock" && <span>{clockZoneLabel(now, language, widget.clockSettings?.timeZone ?? null)}</span>}</div>
            {taskWidget && <div className="progress-circle" style={{ "--progress": `${progress * 3.6}deg` } as CSSProperties}><b>{completed}/{tasks.length}</b></div>}
          </div>
          {widgetContent(widget)}
          {interactiveWidget && <small className={`widget-interaction-hint ${editMode ? "is-active" : ""}`}>{editMode ? t("widget.controlsActive") : t("widget.controlsRequireEdit")}</small>}
          {editMode && !widget.locked && (["n", "s", "e", "w", "ne", "nw", "se", "sw"] as InteractionMode[]).map((edge) => (
            <span className={`resize-handle resize-${edge}`} role="separator" aria-label={t("layout.resizeHandle")} key={edge} onPointerDown={(event) => beginInteraction(widget, edge, event)} onPointerMove={continueInteraction} onPointerUp={finishInteraction} onPointerCancel={finishInteraction} />
          ))}
        </section>;
      })}
    </div>
  );
}

export function calculateLayout(initial: DesktopWidget, mode: InteractionMode, deltaX: number, deltaY: number, bounds: Pick<DOMRect, "width" | "height">, gridSize = DEFAULT_GRID_SIZE): DesktopWidget {
  const margin = SURFACE_MARGIN;
  const limits = widgetSizeLimits(initial.kind, bounds);
  let left = initial.x;
  let top = initial.y;
  let right = initial.x + initial.width;
  let bottom = initial.y + initial.height;
  if (mode === "move") {
    left = clamp(initial.x + deltaX, margin, 1 - margin - initial.width);
    top = clamp(initial.y + deltaY, margin, 1 - margin - initial.height);
    right = left + initial.width;
    bottom = top + initial.height;
  } else {
    if (mode.includes("w")) left = clamp(initial.x + deltaX, Math.max(margin, right - limits.maxWidth), right - limits.minWidth);
    if (mode.includes("e")) right = clamp(initial.x + initial.width + deltaX, left + limits.minWidth, Math.min(1 - margin, left + limits.maxWidth));
    if (mode.includes("n")) top = clamp(initial.y + deltaY, Math.max(margin, bottom - limits.maxHeight), bottom - limits.minHeight);
    if (mode.includes("s")) bottom = clamp(initial.y + initial.height + deltaY, top + limits.minHeight, Math.min(1 - margin, top + limits.maxHeight));
  }
  if (initial.snapToGrid && gridSize > 0) {
    const grid = gridSize;
    if (mode === "move" || mode.includes("w")) left = snap(left, grid);
    if (mode === "move" || mode.includes("n")) top = snap(top, grid);
    if (mode !== "move" && mode.includes("e")) right = snap(right, grid);
    if (mode !== "move" && mode.includes("s")) bottom = snap(bottom, grid);
  }
  const edgeX = 12 / bounds.width;
  const edgeY = 12 / bounds.height;
  if (Math.abs(left - margin) <= edgeX) left = margin;
  if (Math.abs(top - margin) <= edgeY) top = margin;
  if (Math.abs(1 - margin - right) <= edgeX) right = 1 - margin;
  if (Math.abs(1 - margin - bottom) <= edgeY) bottom = 1 - margin;
  if (mode === "move") {
    left = clamp(left, margin, 1 - margin - initial.width);
    top = clamp(top, margin, 1 - margin - initial.height);
    right = left + initial.width;
    bottom = top + initial.height;
  } else {
    left = clamp(left, Math.max(margin, right - limits.maxWidth), right - limits.minWidth);
    top = clamp(top, Math.max(margin, bottom - limits.maxHeight), bottom - limits.minHeight);
    right = clamp(right, left + limits.minWidth, Math.min(1 - margin, left + limits.maxWidth));
    bottom = clamp(bottom, top + limits.minHeight, Math.min(1 - margin, top + limits.maxHeight));
  }
  return { ...initial, x: round(left), y: round(top), width: round(right - left), height: round(bottom - top) };
}

function widgetTitle(kind: WidgetKind, t: (key: TranslationKey) => string) {
  const keys: Record<WidgetKind, TranslationKey> = {
    focus: "widget.focusTitle", kanban: "widget.boardTitle", pomodoro: "widget.pomodoroTitle", clock: "widget.clockTitle", date: "widget.dateTitle",
    dailyPoem: "widget.dailyPoemTitle", dailyVerse: "widget.dailyVerseTitle", dailyHadith: "widget.dailyHadithTitle",
  };
  return t(keys[kind]);
}

function pomodoroRemaining(state: PomodoroState | undefined, now: Date) {
  if (!state) return 1_500;
  if (!state.running || state.endsAt === null) return state.remainingSeconds;
  return Math.max(0, state.endsAt - Math.floor(now.getTime() / 1_000));
}

function formatDuration(seconds: number) { return `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(seconds % 60).padStart(2, "0")}`; }
function resolveLocale(language: LanguagePreference) { return language === "system" ? navigator.language : language === "tr" ? "tr-TR" : "en-US"; }
function defaultClockSettings(): ClockWidgetSettings { return { version: 1, style: "digital", hourFormat: "system", timeZone: null, showSeconds: true, showDate: true, showWeekday: true }; }
function formatClock(date: Date, language: LanguagePreference, settings: ClockWidgetSettings) {
  return new Intl.DateTimeFormat(resolveLocale(language), {
    hour: "2-digit",
    minute: "2-digit",
    second: settings.showSeconds ? "2-digit" : undefined,
    hour12: settings.hourFormat === "system" ? undefined : settings.hourFormat === "hour12",
    timeZone: settings.timeZone ?? undefined,
  }).format(date);
}
function formatClockDate(date: Date, language: LanguagePreference, settings: ClockWidgetSettings) {
  if (!settings.showDate && !settings.showWeekday) return "";
  return new Intl.DateTimeFormat(resolveLocale(language), {
    day: settings.showDate ? "2-digit" : undefined,
    month: settings.showDate ? "long" : undefined,
    year: settings.showDate ? "numeric" : undefined,
    weekday: settings.showWeekday ? "long" : undefined,
    timeZone: settings.timeZone ?? undefined,
  }).format(date);
}
function clockZoneLabel(date: Date, language: LanguagePreference, timeZone: string | null) {
  const parts = new Intl.DateTimeFormat(resolveLocale(language), { timeZone: timeZone ?? undefined, timeZoneName: "short" }).formatToParts(date);
  return parts.find((part) => part.type === "timeZoneName")?.value ?? timeZone ?? "";
}
function clockHandAngles(date: Date, timeZone: string | null) {
  const values = Object.fromEntries(new Intl.DateTimeFormat("en-US", {
    timeZone: timeZone ?? undefined,
    hour: "numeric",
    minute: "numeric",
    second: "numeric",
    hourCycle: "h23",
  }).formatToParts(date).map((part) => [part.type, part.value]));
  const hour = Number(values.hour ?? 0);
  const minute = Number(values.minute ?? 0);
  const second = Number(values.second ?? 0);
  return { hour: (hour % 12) * 30 + minute * .5, minute: minute * 6 + second * .1, second: second * 6 };
}
async function openExternal(url: string) {
  if (isTauriRuntime()) await openUrl(url);
  else window.open(url, "_blank", "noopener,noreferrer");
}
function snap(value: number, grid: number) { return Math.round(value / grid) * grid; }
function clamp(value: number, minimum: number, maximum: number) { return Math.min(maximum, Math.max(minimum, value)); }
function round(value: number) { return Math.round(value * 1_000_000) / 1_000_000; }
function nextStatus(status: TaskStatus): TaskStatus { return status === "todo" ? "inProgress" : status === "inProgress" ? "done" : "todo"; }
