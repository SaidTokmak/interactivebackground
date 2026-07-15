import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, BackgroundSettings, DesktopHostStatus, DesktopWidget, MonitorInfo, PomodoroAction, PomodoroState, Task, TaskStatus, WallpaperTemplate, WidgetKind, WidgetLayout } from "./types";

let browserTasks: Task[] = [
  { id: 1, title: "Rust ownership notlarını bitir", status: "done", scheduledFor: "09:30" },
  { id: 2, title: "Wallpaper pencere prototipi", status: "done", scheduledFor: "11:00" },
  { id: 3, title: "SQLite görev modelini kur", status: "inProgress", scheduledFor: "14:00" },
  { id: 4, title: "30 dk yürüyüş", status: "todo", scheduledFor: "18:30" },
];

export function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}

export async function listTasks(): Promise<Task[]> {
  if (isTauriRuntime()) return invoke<Task[]>("list_tasks");
  return browserTasks.map((task) => ({ ...task }));
}

export async function createTask(title: string, scheduledFor: string | null): Promise<Task> {
  if (isTauriRuntime()) {
    return invoke<Task>("create_task", { title, scheduledFor });
  }

  const task: Task = {
    id: Math.max(0, ...browserTasks.map((item) => item.id)) + 1,
    title,
    status: "todo",
    scheduledFor,
  };
  browserTasks = [...browserTasks, task];
  return task;
}

export async function toggleTask(id: number): Promise<Task> {
  if (isTauriRuntime()) return invoke<Task>("toggle_task", { id });

  const current = browserTasks.find((task) => task.id === id);
  if (!current) throw new Error(`${id} numaralı görev bulunamadı.`);
  const updated: Task = { ...current, status: current.status === "done" ? "todo" : "done" };
  browserTasks = browserTasks.map((task) => (task.id === id ? updated : task));
  return updated;
}

export async function moveTask(id: number, status: TaskStatus): Promise<Task> {
  if (isTauriRuntime()) return invoke<Task>("move_task", { id, status });

  const current = browserTasks.find((task) => task.id === id);
  if (!current) throw new Error(`${id} numaralı görev bulunamadı.`);
  const updated = { ...current, status };
  browserTasks = browserTasks.map((task) => (task.id === id ? updated : task));
  return updated;
}

export async function deleteTask(id: number): Promise<void> {
  if (isTauriRuntime()) {
    await invoke("delete_task", { id });
    return;
  }
  browserTasks = browserTasks.filter((task) => task.id !== id);
}

export async function showWallpaper(): Promise<DesktopHostStatus> {
  if (isTauriRuntime()) return invoke<DesktopHostStatus>("show_wallpaper");
  return { attached: false, mode: "window", warning: null };
}

export async function hideWallpaper(): Promise<void> {
  if (isTauriRuntime()) await invoke("hide_wallpaper");
}

export async function recordInteractionActivity(): Promise<void> {
  if (isTauriRuntime()) await invoke("record_interaction_activity");
}

export async function getDesktopHostStatus(): Promise<DesktopHostStatus> {
  if (isTauriRuntime()) return invoke<DesktopHostStatus>("desktop_host_status");
  return { attached: false, mode: "window", warning: null };
}

export async function listMonitors(): Promise<MonitorInfo[]> {
  if (isTauriRuntime()) return invoke<MonitorInfo[]>("list_monitors");
  return [{
    id: `browser:0:0:${window.screen.width}x${window.screen.height}`,
    name: "Tarayıcı ekranı",
    x: 0,
    y: 0,
    width: window.screen.width,
    height: window.screen.height,
    scaleFactor: window.devicePixelRatio,
    isPrimary: true,
  }];
}

export async function getSettings(): Promise<AppSettings> {
  if (isTauriRuntime()) return invoke<AppSettings>("get_settings");
  return { ...browserSettings };
}

export async function updateSettings(settings: AppSettings): Promise<AppSettings> {
  if (isTauriRuntime()) return invoke<AppSettings>("update_settings", { settings });
  browserSettings = { ...settings };
  return { ...browserSettings };
}

export async function getBackgroundSettings(monitorId: string | null): Promise<BackgroundSettings> {
  if (isTauriRuntime()) return invoke<BackgroundSettings>("get_background_settings", { monitorId });
  return { ...(browserBackgrounds.get(monitorId ?? "__primary__") ?? defaultBackground(monitorId)) };
}

export async function updateBackgroundSettings(settings: BackgroundSettings): Promise<BackgroundSettings> {
  if (isTauriRuntime()) {
    return invoke<BackgroundSettings>("update_background_settings", { settings });
  }
  browserBackgrounds.set(settings.monitorId ?? "__primary__", { ...settings });
  return { ...settings };
}

export async function chooseBackgroundImage(filterName: string): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  return invoke<string | null>("choose_background_image", { filterName });
}

export async function getWidgetLayout(monitorId: string | null, template: WallpaperTemplate): Promise<WidgetLayout> {
  if (isTauriRuntime()) return invoke<WidgetLayout>("get_widget_layout", { monitorId, template });
  const key = widgetLayoutKey(monitorId, template);
  return { ...(browserWidgetLayouts.get(key) ?? defaultWidgetLayout(monitorId, template)) };
}

export async function updateWidgetLayout(layout: WidgetLayout): Promise<WidgetLayout> {
  if (isTauriRuntime()) return invoke<WidgetLayout>("update_widget_layout", { layout });
  browserWidgetLayouts.set(widgetLayoutKey(layout.monitorId, layout.template), { ...layout });
  return { ...layout };
}

export async function resetWidgetLayout(monitorId: string | null, template: WallpaperTemplate): Promise<WidgetLayout> {
  if (isTauriRuntime()) return invoke<WidgetLayout>("reset_widget_layout", { monitorId, template });
  browserWidgetLayouts.delete(widgetLayoutKey(monitorId, template));
  return defaultWidgetLayout(monitorId, template);
}

export async function listDesktopWidgets(monitorId: string | null): Promise<DesktopWidget[]> {
  if (isTauriRuntime()) return invoke<DesktopWidget[]>("list_desktop_widgets", { monitorId });
  return browserWidgets.filter((widget) => widget.monitorId === monitorId).sort((a, b) => a.sortOrder - b.sortOrder).map((widget) => ({ ...widget }));
}

export async function addDesktopWidget(monitorId: string | null, kind: WidgetKind): Promise<DesktopWidget> {
  if (isTauriRuntime()) return invoke<DesktopWidget>("add_desktop_widget", { monitorId, kind });
  if (browserWidgets.filter((widget) => widget.monitorId === monitorId).length >= 12) throw new Error("Bir monitörde en fazla 12 widget kullanılabilir.");
  const id = Math.max(0, ...browserWidgets.map((widget) => widget.id)) + 1;
  const sortOrder = browserWidgets.filter((widget) => widget.monitorId === monitorId).length;
  const widget = defaultDesktopWidget(monitorId, kind, id, sortOrder);
  browserWidgets.push(widget);
  if (kind === "pomodoro") browserPomodoros.set(id, defaultPomodoro(id));
  return { ...widget };
}

export async function updateDesktopWidget(widget: DesktopWidget): Promise<DesktopWidget> {
  if (isTauriRuntime()) return invoke<DesktopWidget>("update_desktop_widget", { widget });
  const index = browserWidgets.findIndex((item) => item.id === widget.id);
  if (index < 0) throw new Error("Widget bulunamadı.");
  browserWidgets[index] = { ...widget };
  return { ...widget };
}

export async function duplicateDesktopWidget(id: number): Promise<DesktopWidget> {
  if (isTauriRuntime()) return invoke<DesktopWidget>("duplicate_desktop_widget", { id });
  const original = browserWidgets.find((widget) => widget.id === id);
  if (!original) throw new Error("Widget bulunamadı.");
  const duplicate = await addDesktopWidget(original.monitorId, original.kind);
  const updated = {
    ...duplicate,
    x: Math.min(original.x + 0.025, 0.985 - original.width),
    y: Math.min(original.y + 0.025, 0.985 - original.height),
    width: original.width,
    height: original.height,
    snapToGrid: original.snapToGrid,
  };
  return updateDesktopWidget(updated);
}

export async function deleteDesktopWidget(id: number): Promise<void> {
  if (isTauriRuntime()) await invoke("delete_desktop_widget", { id });
  else {
    browserWidgets = browserWidgets.filter((widget) => widget.id !== id);
    browserPomodoros.delete(id);
  }
}

export async function reorderDesktopWidgets(monitorId: string | null, orderedIds: number[]): Promise<DesktopWidget[]> {
  if (isTauriRuntime()) return invoke<DesktopWidget[]>("reorder_desktop_widgets", { monitorId, orderedIds });
  browserWidgets = browserWidgets.map((widget) => {
    const index = widget.monitorId === monitorId ? orderedIds.indexOf(widget.id) : -1;
    return index >= 0 ? { ...widget, sortOrder: index } : widget;
  });
  return listDesktopWidgets(monitorId);
}

export async function getPomodoroState(widgetId: number): Promise<PomodoroState> {
  if (isTauriRuntime()) return invoke<PomodoroState>("get_pomodoro_state", { widgetId });
  const state = browserPomodoros.get(widgetId) ?? defaultPomodoro(widgetId);
  return normalizeBrowserPomodoro(state);
}

export async function updatePomodoro(widgetId: number, action: PomodoroAction): Promise<PomodoroState> {
  if (isTauriRuntime()) return invoke<PomodoroState>("update_pomodoro", { widgetId, action });
  let state = await getPomodoroState(widgetId);
  const now = Math.floor(Date.now() / 1000);
  if (action === "start") state = { ...state, running: true, endsAt: now + Math.max(1, state.remainingSeconds) };
  if (action === "pause") state = { ...state, running: false, endsAt: null };
  if (action === "reset") state = { ...state, mode: "work", remainingSeconds: state.workMinutes * 60, running: false, endsAt: null };
  if (action === "skip") {
    const mode = state.mode === "work" ? "break" : "work";
    state = { ...state, mode, remainingSeconds: (mode === "work" ? state.workMinutes : state.breakMinutes) * 60, running: false, endsAt: null };
  }
  browserPomodoros.set(widgetId, state);
  return { ...state };
}

export async function configurePomodoro(widgetId: number, workMinutes: number, breakMinutes: number): Promise<PomodoroState> {
  if (isTauriRuntime()) return invoke<PomodoroState>("configure_pomodoro", { widgetId, workMinutes, breakMinutes });
  const state: PomodoroState = { widgetId, mode: "work", workMinutes, breakMinutes, remainingSeconds: workMinutes * 60, running: false, endsAt: null };
  browserPomodoros.set(widgetId, state);
  return { ...state };
}

let browserSettings: AppSettings = {
  template: "focus",
  opacity: 82,
  editMode: false,
  monitorId: null,
  autoCalmMinutes: 5,
  theme: "system",
  language: "system",
};

const browserBackgrounds = new Map<string, BackgroundSettings>();
const browserWidgetLayouts = new Map<string, WidgetLayout>();
let browserWidgets: DesktopWidget[] = [defaultDesktopWidget(null, "focus", 1, 0)];
const browserPomodoros = new Map<number, PomodoroState>();

function defaultBackground(monitorId: string | null): BackgroundSettings {
  return {
    monitorId,
    source: "preset",
    preset: "foldedHorizon",
    customPath: null,
    fit: "cover",
    overlay: 16,
    blur: 0,
  };
}

function widgetLayoutKey(monitorId: string | null, template: WallpaperTemplate) {
  return `${monitorId ?? "__primary__"}:${template}`;
}

function defaultWidgetLayout(monitorId: string | null, template: WallpaperTemplate): WidgetLayout {
  return {
    monitorId,
    template,
    x: template === "focus" ? 0.62 : 0.52,
    y: 0.16,
    width: template === "focus" ? 0.34 : 0.44,
    height: template === "focus" ? 0.56 : 0.54,
    locked: false,
    snapToGrid: true,
  };
}

function defaultDesktopWidget(monitorId: string | null, kind: WidgetKind, id: number, sortOrder: number): DesktopWidget {
  const frames: Record<WidgetKind, [number, number, number, number]> = {
    focus: [0.62, 0.16, 0.34, 0.56],
    kanban: [0.52, 0.16, 0.44, 0.54],
    pomodoro: [0.05, 0.12, 0.25, 0.34],
    clock: [0.05, 0.54, 0.22, 0.20],
    date: [0.30, 0.72, 0.25, 0.18],
  };
  const [baseX, baseY, width, height] = frames[kind];
  const offset = (sortOrder % 6) * 0.025;
  return { id, monitorId, kind, x: Math.min(baseX + offset, 0.985 - width), y: Math.min(baseY + offset, 0.985 - height), width, height, locked: false, snapToGrid: true, visible: true, sortOrder };
}

function defaultPomodoro(widgetId: number): PomodoroState {
  return { widgetId, mode: "work", workMinutes: 25, breakMinutes: 5, remainingSeconds: 1500, running: false, endsAt: null };
}

function normalizeBrowserPomodoro(state: PomodoroState): PomodoroState {
  if (!state.running || state.endsAt === null) return { ...state };
  const remaining = state.endsAt - Math.floor(Date.now() / 1000);
  if (remaining > 0) return { ...state, remainingSeconds: remaining };
  const mode: PomodoroState["mode"] = state.mode === "work" ? "break" : "work";
  const next = { ...state, mode, remainingSeconds: (mode === "work" ? state.workMinutes : state.breakMinutes) * 60, running: false, endsAt: null };
  browserPomodoros.set(state.widgetId, next);
  return { ...next };
}
