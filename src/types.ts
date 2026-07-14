export type TaskStatus = "todo" | "inProgress" | "done";

export type Task = {
  id: number;
  title: string;
  status: TaskStatus;
  scheduledFor: string | null;
};

export type WallpaperTemplate = "focus" | "kanban";

export type AppSettings = {
  template: WallpaperTemplate;
  opacity: number;
  editMode: boolean;
  monitorId: string | null;
  autoCalmMinutes: number | null;
};

export type MonitorInfo = {
  id: string;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactor: number;
  isPrimary: boolean;
};

export type DesktopHostStatus = {
  attached: boolean;
  mode: "workerW" | "interaction" | "fallback" | "window";
  warning: string | null;
};
