export type TaskStatus = "todo" | "inProgress" | "done";

export type Task = {
  id: number;
  title: string;
  status: TaskStatus;
  scheduledFor: string | null;
};

export type WallpaperTemplate = "focus" | "kanban";
export type ThemePreference = "system" | "light" | "dark";
export type LanguagePreference = "system" | "tr" | "en";
export type BackgroundSource = "preset" | "custom";
export type BackgroundPreset = "foldedHorizon" | "midnight" | "graphite" | "ember" | "porcelain" | "arctic" | "linen" | "morningMist";
export type BackgroundFit = "cover" | "contain" | "stretch";

export type BackgroundSettings = {
  monitorId: string | null;
  source: BackgroundSource;
  preset: BackgroundPreset;
  customPath: string | null;
  fit: BackgroundFit;
  overlay: number;
  blur: number;
};

export type WidgetLayout = {
  monitorId: string | null;
  template: WallpaperTemplate;
  x: number;
  y: number;
  width: number;
  height: number;
  locked: boolean;
  snapToGrid: boolean;
};

export type WidgetKind = "focus" | "kanban" | "pomodoro" | "clock" | "date" | "dailyPoem" | "dailyVerse" | "dailyHadith";
export type WidgetPackageSource = "core" | "bundledStore";

export type WidgetPackage = {
  kind: WidgetKind;
  source: WidgetPackageSource;
  version: string;
  installed: boolean;
  minimumWidth: number;
  minimumHeight: number;
  permissions: string[];
};

export type ClockStyle = "digital" | "analog";
export type ClockHourFormat = "system" | "hour12" | "hour24";

export type ClockWidgetSettings = {
  version: 1;
  style: ClockStyle;
  hourFormat: ClockHourFormat;
  timeZone: string | null;
  showSeconds: boolean;
  showDate: boolean;
  showWeekday: boolean;
};

export type DesktopWidget = {
  id: number;
  monitorId: string | null;
  kind: WidgetKind;
  x: number;
  y: number;
  width: number;
  height: number;
  locked: boolean;
  snapToGrid: boolean;
  visible: boolean;
  sortOrder: number;
  clockSettings: ClockWidgetSettings | null;
};

export type PomodoroMode = "work" | "break";
export type PomodoroAction = "start" | "pause" | "reset" | "skip" | "complete";

export type PomodoroState = {
  widgetId: number;
  mode: PomodoroMode;
  workMinutes: number;
  breakMinutes: number;
  remainingSeconds: number;
  running: boolean;
  endsAt: number | null;
};

export type PomodoroPreferences = {
  notificationsEnabled: boolean;
  soundEnabled: boolean;
  soundVolume: number;
};

export type PomodoroCompletion = {
  widgetId: number;
  completedMode: PomodoroMode;
  state: PomodoroState;
};

export type AppSettings = {
  template: WallpaperTemplate;
  opacity: number;
  editMode: boolean;
  monitorId: string | null;
  autoCalmMinutes: number | null;
  theme: ThemePreference;
  language: LanguagePreference;
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
  visible: boolean;
  mode: "workerW" | "interaction" | "fallback" | "window";
  warning: string | null;
};

export type StarterLayout = "focus" | "planning" | "blank";

export type OnboardingPreferences = {
  language: LanguagePreference;
  theme: ThemePreference;
  monitorId: string | null;
  backgroundPreset: BackgroundPreset;
  starterLayout: StarterLayout;
};

export type OnboardingStatus = {
  completed: boolean;
};
