import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, DesktopHostStatus, MonitorInfo, Task, TaskStatus } from "./types";

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

let browserSettings: AppSettings = {
  template: "focus",
  opacity: 82,
  editMode: false,
  monitorId: null,
  autoCalmMinutes: 5,
  theme: "system",
};
