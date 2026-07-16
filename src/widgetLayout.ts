import type { DesktopWidget, MonitorInfo, WidgetKind } from "./types";

export type LayoutViewport = { width: number; height: number };
export type InteractionMode = "move" | "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

export const SURFACE_MARGIN = 0.015;
export const DEFAULT_GRID_SIZE = 0.01;
export const WIDGET_GAP = 0.005;

export function monitorLayoutViewport(monitor: MonitorInfo): LayoutViewport {
  const scale = monitor.scaleFactor > 0 ? monitor.scaleFactor : 1;
  return { width: monitor.width / scale, height: monitor.height / scale };
}

const SIZE_LIMITS: Record<WidgetKind, [number, number, number, number, number, number]> = {
  focus: [0.10, 0.14, 0.78, 0.78, 240, 200],
  kanban: [0.10, 0.14, 0.78, 0.78, 240, 200],
  pomodoro: [0.08, 0.12, 0.50, 0.62, 190, 170],
  clock: [0.06, 0.08, 0.46, 0.42, 140, 95],
  date: [0.07, 0.08, 0.52, 0.42, 160, 95],
  dailyPoem: [0.10, 0.12, 0.58, 0.66, 215, 180],
  dailyVerse: [0.11, 0.13, 0.62, 0.70, 230, 190],
  dailyHadith: [0.11, 0.12, 0.60, 0.64, 230, 180],
};

export function widgetSizeLimits(kind: WidgetKind, viewport: LayoutViewport) {
  const [minWidth, minHeight, maxWidth, maxHeight, minPixelsX, minPixelsY] = SIZE_LIMITS[kind];
  return {
    minWidth: Math.min(maxWidth, Math.max(minWidth, minPixelsX / viewport.width)),
    minHeight: Math.min(maxHeight, Math.max(minHeight, minPixelsY / viewport.height)),
    maxWidth,
    maxHeight,
  };
}

export function widgetsOverlap(candidate: DesktopWidget, other: DesktopWidget, gap = WIDGET_GAP) {
  if (!candidate.visible || !other.visible || candidate.id === other.id) return false;
  return candidate.x < other.x + other.width + gap
    && candidate.x + candidate.width + gap > other.x
    && candidate.y < other.y + other.height + gap
    && candidate.y + candidate.height + gap > other.y;
}

export function hasWidgetCollision(candidate: DesktopWidget, widgets: DesktopWidget[]) {
  return widgets.some((widget) => widgetsOverlap(candidate, widget));
}

export function calculateLayout(initial: DesktopWidget, mode: InteractionMode, deltaX: number, deltaY: number, bounds: LayoutViewport, gridSize = DEFAULT_GRID_SIZE): DesktopWidget {
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
    if (mode === "move" || mode.includes("w")) left = snap(left, gridSize);
    if (mode === "move" || mode.includes("n")) top = snap(top, gridSize);
    if (mode !== "move" && mode.includes("e")) right = snap(right, gridSize);
    if (mode !== "move" && mode.includes("s")) bottom = snap(bottom, gridSize);
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

function snap(value: number, grid: number) { return Math.round(value / grid) * grid; }
function clamp(value: number, minimum: number, maximum: number) { return Math.min(maximum, Math.max(minimum, value)); }
function round(value: number) { return Math.round(value * 1_000_000) / 1_000_000; }
