import { useEffect, useRef, useState, type CSSProperties, type PointerEvent as ReactPointerEvent } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { BackgroundSettings, LanguagePreference, Task, TaskStatus, WallpaperTemplate, WidgetLayout } from "./types";
import appIcon from "./assets/interactivebackground-icon.png";
import { useI18n } from "./i18n";
import { isTauriRuntime } from "./taskApi";

type Props = {
  tasks: Task[];
  template: WallpaperTemplate;
  editMode: boolean;
  opacity: number;
  language: LanguagePreference;
  background: BackgroundSettings;
  layout: WidgetLayout;
  actual?: boolean;
  onToggle: (id: number) => void;
  onMove: (id: number, status: TaskStatus) => void;
  onLayoutChange: (layout: WidgetLayout) => void;
};

type InteractionMode = "move" | "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

type ActiveInteraction = {
  pointerId: number;
  mode: InteractionMode;
  startX: number;
  startY: number;
  initial: WidgetLayout;
  latest: WidgetLayout;
  bounds: DOMRect;
};

export function WallpaperSurface({ tasks, template, editMode, opacity, language, background, layout, actual = false, onToggle, onMove, onLayoutChange }: Props) {
  const { t, formatDate } = useI18n(language);
  const surfaceRef = useRef<HTMLDivElement>(null);
  const interactionRef = useRef<ActiveInteraction | null>(null);
  const [liveLayout, setLiveLayout] = useState(layout);
  useEffect(() => {
    if (!interactionRef.current) setLiveLayout(layout);
  }, [layout]);
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
  const widgetStyle = {
    left: `${liveLayout.x * 100}%`,
    top: `${liveLayout.y * 100}%`,
    width: `${liveLayout.width * 100}%`,
    height: `${liveLayout.height * 100}%`,
    backgroundColor: `color-mix(in srgb, var(--widget) ${opacity}%, transparent)`,
  } as CSSProperties;

  function beginInteraction(mode: InteractionMode, event: ReactPointerEvent<HTMLElement>) {
    if (!editMode || liveLayout.locked || !surfaceRef.current) return;
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    interactionRef.current = {
      pointerId: event.pointerId,
      mode,
      startX: event.clientX,
      startY: event.clientY,
      initial: liveLayout,
      latest: liveLayout,
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
    setLiveLayout(next);
  }

  function finishInteraction(event: ReactPointerEvent<HTMLElement>) {
    const active = interactionRef.current;
    if (!active || active.pointerId !== event.pointerId) return;
    interactionRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    onLayoutChange(active.latest);
  }

  return (
    <div className={`desktop-preview ${actual ? "actual-surface" : ""}`} ref={surfaceRef}>
      <div className={`desktop-background preset-${background.preset} ${customImage ? "custom-background" : ""}`} style={backgroundStyle} />
      <div className="desktop-overlay" style={{ backgroundColor: `rgba(5, 8, 18, ${background.overlay / 100})` }} />
      {editMode && liveLayout.snapToGrid && <div className="layout-grid" />}
      <div className="desktop-topline">
        <span className="desktop-brand"><img src={appIcon} alt="" aria-hidden="true" />interactivebackground</span>
        <span className="desktop-mode">⌁ {editMode ? t("wallpaper.mode.edit") : t("wallpaper.mode.calm")}</span>
      </div>
      <div className="desktop-icon"><span>▱</span>{t("desktop.projects")}</div>
      <div className="desktop-icon second"><span>♲</span>{t("desktop.trash")}</div>

      <section className={`wallpaper-widget ${actual ? "actual-widget" : ""} ${editMode ? "editing" : ""} ${liveLayout.locked ? "layout-locked" : ""}`} style={widgetStyle}>
        <div className="widget-header widget-drag-handle" onPointerDown={(event) => beginInteraction("move", event)} onPointerMove={continueInteraction} onPointerUp={finishInteraction} onPointerCancel={finishInteraction}>
          <div><h3>{template === "focus" ? t("widget.focusTitle") : t("widget.boardTitle")}</h3><span>{formatDate(new Date(), "long")}</span></div>
          <div className="progress-circle" style={{ "--progress": `${progress * 3.6}deg` } as CSSProperties}>
            <b>{completed}/{tasks.length}</b>
          </div>
        </div>

        {editMode && liveLayout.locked && <span className="widget-lock-badge">{t("layout.lockedBadge")}</span>}

        {template === "focus" ? (
          <>
            <div className="widget-tasks">
              {tasks.slice(0, 6).map((task) => (
                <label className={task.status === "done" ? "done" : ""} key={task.id}>
                  <input type="checkbox" checked={task.status === "done"} onChange={() => onToggle(task.id)} />
                  <span>{task.title}</span>
                  <time>{task.scheduledFor}</time>
                </label>
              ))}
            </div>
            <div className="focus-action">
              <span>{nextTask ? t("widget.next", { title: nextTask.title }) : t("widget.allDone")}</span>
              <button>{t("widget.focusButton")}</button>
            </div>
          </>
        ) : (
          <div className="kanban-board">
            {columns.map((column) => (
              <div className="kanban-column" key={column.status}>
                <span>{column.label}</span>
                {tasks.filter((task) => task.status === column.status).map((task) => (
                  <button className="kanban-card" key={task.id} onClick={() => onMove(task.id, nextStatus(column.status))}>
                    {task.title}
                  </button>
                ))}
              </div>
            ))}
          </div>
        )}
        {editMode && !liveLayout.locked && (["n", "s", "e", "w", "ne", "nw", "se", "sw"] as InteractionMode[]).map((edge) => (
          <span className={`resize-handle resize-${edge}`} key={edge} onPointerDown={(event) => beginInteraction(edge, event)} onPointerMove={continueInteraction} onPointerUp={finishInteraction} onPointerCancel={finishInteraction} />
        ))}
      </section>
    </div>
  );
}

export function calculateLayout(
  initial: WidgetLayout,
  mode: InteractionMode,
  deltaX: number,
  deltaY: number,
  bounds: Pick<DOMRect, "width" | "height">,
): WidgetLayout {
  const margin = 0.015;
  const minWidth = Math.min(0.45, Math.max(0.18, 280 / bounds.width));
  const minHeight = Math.min(0.62, Math.max(0.20, 250 / bounds.height));
  const maxWidth = 0.78;
  const maxHeight = 0.78;
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
    if (mode.includes("w")) left = clamp(initial.x + deltaX, Math.max(margin, right - maxWidth), right - minWidth);
    if (mode.includes("e")) right = clamp(initial.x + initial.width + deltaX, left + minWidth, Math.min(1 - margin, left + maxWidth));
    if (mode.includes("n")) top = clamp(initial.y + deltaY, Math.max(margin, bottom - maxHeight), bottom - minHeight);
    if (mode.includes("s")) bottom = clamp(initial.y + initial.height + deltaY, top + minHeight, Math.min(1 - margin, top + maxHeight));
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
    const width = initial.width;
    const height = initial.height;
    left = clamp(left, margin, 1 - margin - width);
    top = clamp(top, margin, 1 - margin - height);
    right = left + width;
    bottom = top + height;
  } else {
    left = clamp(left, Math.max(margin, right - maxWidth), right - minWidth);
    top = clamp(top, Math.max(margin, bottom - maxHeight), bottom - minHeight);
    right = clamp(right, left + minWidth, Math.min(1 - margin, left + maxWidth));
    bottom = clamp(bottom, top + minHeight, Math.min(1 - margin, top + maxHeight));
  }

  return {
    ...initial,
    x: round(left),
    y: round(top),
    width: round(right - left),
    height: round(bottom - top),
  };
}

function snap(value: number, grid: number) {
  return Math.round(value / grid) * grid;
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(maximum, Math.max(minimum, value));
}

function round(value: number) {
  return Math.round(value * 1_000_000) / 1_000_000;
}

function nextStatus(status: TaskStatus): TaskStatus {
  if (status === "todo") return "inProgress";
  if (status === "inProgress") return "done";
  return "todo";
}
