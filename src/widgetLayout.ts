import type { DesktopWidget, MonitorInfo, WidgetKind } from "./types";

export type LayoutViewport = { width: number; height: number };

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
