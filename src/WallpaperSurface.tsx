import { useEffect, useRef, useState, type CSSProperties, type PointerEvent as ReactPointerEvent, type ReactNode } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { BackgroundSettings, DesktopWidget, LanguagePreference, PomodoroAction, PomodoroState, Task, TaskStatus, WidgetKind } from "./types";
import appIcon from "./assets/interactivebackground-icon.png";
import { useI18n } from "./i18n";
import type { TranslationKey } from "./i18n/locales/en";
import { getDailyContent } from "./dailyContent";
import { isTauriRuntime } from "./taskApi";

type Props = {
  tasks: Task[];
  widgets: DesktopWidget[];
  pomodoros: Record<number, PomodoroState>;
  editMode: boolean;
  opacity: number;
  language: LanguagePreference;
  background: BackgroundSettings;
  actual?: boolean;
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
  bounds: DOMRect;
};

export function WallpaperSurface({ tasks, widgets, pomodoros, editMode, opacity, language, background, actual = false, onToggle, onMove, onWidgetChange, onPomodoroAction }: Props) {
  const { t, formatDate } = useI18n(language);
  const surfaceRef = useRef<HTMLDivElement>(null);
  const interactionRef = useRef<ActiveInteraction | null>(null);
  const completedPomodorosRef = useRef(new Set<string>());
  const [liveWidgets, setLiveWidgets] = useState(widgets);
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    if (!interactionRef.current) setLiveWidgets(widgets);
  }, [widgets]);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1_000);
    return () => window.clearInterval(timer);
  }, []);
  useEffect(() => {
    const timestamp = Math.floor(now.getTime() / 1_000);
    Object.values(pomodoros).forEach((state) => {
      if (!state.running || state.endsAt === null || state.endsAt > timestamp) return;
      const completionKey = `${state.widgetId}:${state.endsAt}`;
      if (completedPomodorosRef.current.has(completionKey)) return;
      completedPomodorosRef.current.add(completionKey);
      onPomodoroAction(state.widgetId, "complete");
    });
  }, [now, onPomodoroAction, pomodoros]);

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
      active.bounds,
    );
    active.latest = next;
    setLiveWidgets((current) => current.map((widget) => widget.id === next.id ? next : widget));
  }

  function finishInteraction(event: ReactPointerEvent<HTMLElement>) {
    const active = interactionRef.current;
    if (!active || active.pointerId !== event.pointerId) return;
    interactionRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    onWidgetChange(active.latest);
  }

  function widgetContent(widget: DesktopWidget): ReactNode {
    if (widget.kind === "focus") {
      return <>
        <div className="widget-tasks">
          {tasks.slice(0, 6).map((task) => (
            <label className={task.status === "done" ? "done" : ""} key={task.id}>
              <input type="checkbox" checked={task.status === "done"} onChange={() => onToggle(task.id)} />
              <span>{task.title}</span><time>{task.scheduledFor}</time>
            </label>
          ))}
        </div>
        <div className="focus-action"><span>{nextTask ? t("widget.next", { title: nextTask.title }) : t("widget.allDone")}</span><button>{t("widget.focusButton")}</button></div>
      </>;
    }
    if (widget.kind === "kanban") {
      return <div className="kanban-board">
        {columns.map((column) => (
          <div className="kanban-column" key={column.status}>
            <span>{column.label}</span>
            {tasks.filter((task) => task.status === column.status).map((task) => (
              <button className="kanban-card" key={task.id} onClick={() => onMove(task.id, nextStatus(column.status))}>{task.title}</button>
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
          <button onClick={() => onPomodoroAction(widget.id, state?.running ? "pause" : "start")}>{state?.running ? t("pomodoro.pause") : t("pomodoro.start")}</button>
          <button onClick={() => onPomodoroAction(widget.id, "reset")}>{t("pomodoro.reset")}</button>
          <button onClick={() => onPomodoroAction(widget.id, "skip")}>{t("pomodoro.skip")}</button>
        </div>
      </div>;
    }
    if (widget.kind === "clock") {
      return <div className="clock-body"><strong>{formatClock(now, language)}</strong><span>{formatDate(now, "long")}</span></div>;
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
            {content.originalSourceUrl && <button type="button" onClick={() => void openExternal(content.originalSourceUrl!)}>{t("daily.arabicSource")}</button>}
            <button type="button" onClick={() => void openExternal(content.sourceUrl)}>{t("daily.source")}</button>
          </div>
        </div>
        <small className="daily-license">{content.license}</small>
      </div>;
    }
    return <div className="date-body"><strong>{now.toLocaleDateString(resolveLocale(language), { day: "2-digit" })}</strong><div><span>{now.toLocaleDateString(resolveLocale(language), { month: "long" })}</span><b>{now.toLocaleDateString(resolveLocale(language), { weekday: "long" })}</b></div></div>;
  }

  return (
    <div className={`desktop-preview ${actual ? "actual-surface" : ""}`} ref={surfaceRef}>
      <div className={`desktop-background preset-${background.preset} ${customImage ? "custom-background" : ""}`} style={backgroundStyle} />
      <div className="desktop-overlay" style={{ backgroundColor: `rgba(5, 8, 18, ${background.overlay / 100})` }} />
      {editMode && liveWidgets.some((widget) => widget.visible && widget.snapToGrid) && <div className="layout-grid" />}
      <div className="desktop-topline">
        <span className="desktop-brand"><img src={appIcon} alt="" aria-hidden="true" />interactivebackground</span>
        <span className="desktop-mode">⌁ {editMode ? t("wallpaper.mode.edit") : t("wallpaper.mode.calm")}</span>
      </div>
      <div className="desktop-icon"><span>▱</span>{t("desktop.projects")}</div>
      <div className="desktop-icon second"><span>♲</span>{t("desktop.trash")}</div>

      {liveWidgets.filter((widget) => widget.visible).map((widget) => {
        const style = {
          left: `${widget.x * 100}%`, top: `${widget.y * 100}%`, width: `${widget.width * 100}%`, height: `${widget.height * 100}%`,
          zIndex: 3 + widget.sortOrder,
          backgroundColor: `color-mix(in srgb, var(--widget) ${opacity}%, transparent)`,
        } as CSSProperties;
        const taskWidget = widget.kind === "focus" || widget.kind === "kanban";
        return <section className={`wallpaper-widget widget-${widget.kind} ${actual ? "actual-widget" : ""} ${editMode ? "editing" : ""} ${widget.locked ? "layout-locked" : ""}`} style={style} key={widget.id}>
          <div className="widget-header widget-drag-handle" onPointerDown={(event) => beginInteraction(widget, "move", event)} onPointerMove={continueInteraction} onPointerUp={finishInteraction} onPointerCancel={finishInteraction}>
            <div><h3>{widgetTitle(widget.kind, t)}</h3>{taskWidget && <span>{formatDate(now, "long")}</span>}</div>
            {taskWidget && <div className="progress-circle" style={{ "--progress": `${progress * 3.6}deg` } as CSSProperties}><b>{completed}/{tasks.length}</b></div>}
          </div>
          {editMode && widget.locked && <span className="widget-lock-badge">{t("layout.lockedBadge")}</span>}
          {widgetContent(widget)}
          {editMode && !widget.locked && (["n", "s", "e", "w", "ne", "nw", "se", "sw"] as InteractionMode[]).map((edge) => (
            <span className={`resize-handle resize-${edge}`} key={edge} onPointerDown={(event) => beginInteraction(widget, edge, event)} onPointerMove={continueInteraction} onPointerUp={finishInteraction} onPointerCancel={finishInteraction} />
          ))}
        </section>;
      })}
    </div>
  );
}

export function calculateLayout(initial: DesktopWidget, mode: InteractionMode, deltaX: number, deltaY: number, bounds: Pick<DOMRect, "width" | "height">): DesktopWidget {
  const margin = 0.015;
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
  if (initial.snapToGrid) {
    const grid = 0.025;
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

function widgetSizeLimits(kind: WidgetKind, bounds: Pick<DOMRect, "width" | "height">) {
  const values: Record<WidgetKind, [number, number, number, number, number, number]> = {
    focus: [0.18, 0.20, 0.78, 0.78, 280, 250], kanban: [0.18, 0.20, 0.78, 0.78, 280, 250],
    pomodoro: [0.18, 0.24, 0.50, 0.62, 230, 220], clock: [0.12, 0.14, 0.46, 0.42, 180, 120], date: [0.16, 0.14, 0.52, 0.42, 200, 120],
    dailyPoem: [0.20, 0.24, 0.58, 0.66, 270, 230], dailyVerse: [0.22, 0.26, 0.62, 0.70, 290, 250], dailyHadith: [0.22, 0.24, 0.60, 0.64, 290, 230],
  };
  const [minWidth, minHeight, maxWidth, maxHeight, minPixelsX, minPixelsY] = values[kind];
  return { minWidth: Math.min(maxWidth, Math.max(minWidth, minPixelsX / bounds.width)), minHeight: Math.min(maxHeight, Math.max(minHeight, minPixelsY / bounds.height)), maxWidth, maxHeight };
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
function formatClock(date: Date, language: LanguagePreference) { return new Intl.DateTimeFormat(resolveLocale(language), { hour: "2-digit", minute: "2-digit", second: "2-digit" }).format(date); }
async function openExternal(url: string) {
  if (isTauriRuntime()) await openUrl(url);
  else window.open(url, "_blank", "noopener,noreferrer");
}
function snap(value: number, grid: number) { return Math.round(value / grid) * grid; }
function clamp(value: number, minimum: number, maximum: number) { return Math.min(maximum, Math.max(minimum, value)); }
function round(value: number) { return Math.round(value * 1_000_000) / 1_000_000; }
function nextStatus(status: TaskStatus): TaskStatus { return status === "todo" ? "inProgress" : status === "inProgress" ? "done" : "todo"; }
