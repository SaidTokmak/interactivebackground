import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { cancel } from "@tauri-apps/plugin-notification";
import {
  addDesktopWidget,
  configurePomodoro,
  deleteDesktopWidget,
  duplicateDesktopWidget,
  getPomodoroState,
  isTauriRuntime,
  listDesktopWidgets,
  reorderDesktopWidgets,
  updateDesktopWidget,
  updatePomodoro,
} from "./taskApi";
import type { DesktopWidget, PomodoroAction, PomodoroState, WidgetKind } from "./types";

export function useDesktopWidgets(monitorId: string | null) {
  const [widgets, setWidgets] = useState<DesktopWidget[]>([]);
  const [pomodoros, setPomodoros] = useState<Record<number, PomodoroState>>({});
  const [widgetError, setWidgetError] = useState("");

  const refreshWidgets = useCallback(async () => {
    try {
      const next = await listDesktopWidgets(monitorId);
      setWidgets(next);
      setWidgetError("");
      const pomodoroEntries = await Promise.all(
        next.filter((widget) => widget.kind === "pomodoro").map(async (widget) => [widget.id, await getPomodoroState(widget.id)] as const),
      );
      setPomodoros(Object.fromEntries(pomodoroEntries));
    } catch (reason) {
      setWidgetError(String(reason));
    }
  }, [monitorId]);

  const refreshPomodoros = useCallback(async () => {
    try {
      const currentWidgets = await listDesktopWidgets(monitorId);
      const entries = await Promise.all(
        currentWidgets.filter((widget) => widget.kind === "pomodoro").map(async (widget) => [widget.id, await getPomodoroState(widget.id)] as const),
      );
      setPomodoros(Object.fromEntries(entries));
      setWidgetError("");
    } catch (reason) {
      setWidgetError(String(reason));
    }
  }, [monitorId]);

  useEffect(() => {
    setWidgets([]);
    void refreshWidgets();
    if (!isTauriRuntime()) return;
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];
    void Promise.all([
      listen("desktop-widgets-changed", () => void refreshWidgets()),
      listen("pomodoro-changed", () => void refreshPomodoros()),
    ]).then((stops) => {
      if (disposed) stops.forEach((stop) => stop());
      else unlisteners.push(...stops);
    });
    return () => {
      disposed = true;
      unlisteners.forEach((stop) => stop());
    };
  }, [refreshPomodoros, refreshWidgets]);

  async function addWidget(kind: WidgetKind) {
    try {
      const widget = await addDesktopWidget(monitorId, kind);
      setWidgets((current) => [...current, widget]);
      if (kind === "pomodoro") setPomodoros((current) => ({ ...current, [widget.id]: { widgetId: widget.id, mode: "work", workMinutes: 25, breakMinutes: 5, remainingSeconds: 1500, running: false, endsAt: null } }));
      setWidgetError("");
      return widget;
    } catch (reason) {
      setWidgetError(String(reason));
      return undefined;
    }
  }

  async function saveWidget(widget: DesktopWidget) {
    setWidgets((current) => current.map((item) => item.id === widget.id ? widget : item));
    try {
      const saved = await updateDesktopWidget(widget);
      setWidgets((current) => current.map((item) => item.id === saved.id ? saved : item));
      setWidgetError("");
    } catch (reason) {
      setWidgetError(String(reason));
      await refreshWidgets();
    }
  }

  async function duplicateWidget(id: number) {
    try {
      await duplicateDesktopWidget(id);
      await refreshWidgets();
    } catch (reason) {
      setWidgetError(String(reason));
    }
  }

  async function removeWidget(id: number) {
    try {
      if (isTauriRuntime()) await cancel([id]).catch(() => undefined);
      await deleteDesktopWidget(id);
      setWidgets((current) => current.filter((widget) => widget.id !== id));
      setPomodoros((current) => {
        const next = { ...current };
        delete next[id];
        return next;
      });
      setWidgetError("");
    } catch (reason) {
      setWidgetError(String(reason));
    }
  }

  async function moveWidget(id: number, direction: -1 | 1) {
    const index = widgets.findIndex((widget) => widget.id === id);
    const target = index + direction;
    if (index < 0 || target < 0 || target >= widgets.length) return;
    const next = [...widgets];
    [next[index], next[target]] = [next[target], next[index]];
    setWidgets(next.map((widget, sortOrder) => ({ ...widget, sortOrder })));
    try {
      setWidgets(await reorderDesktopWidgets(monitorId, next.map((widget) => widget.id)));
      setWidgetError("");
    } catch (reason) {
      setWidgetError(String(reason));
      await refreshWidgets();
    }
  }

  async function controlPomodoro(widgetId: number, action: PomodoroAction) {
    try {
      if (isTauriRuntime() && action !== "complete") await cancel([widgetId]).catch(() => undefined);
      const state = await updatePomodoro(widgetId, action);
      setPomodoros((current) => ({ ...current, [widgetId]: state }));
      setWidgetError("");
    } catch (reason) {
      setWidgetError(String(reason));
    }
  }

  async function savePomodoroDurations(widgetId: number, workMinutes: number, breakMinutes: number) {
    try {
      if (isTauriRuntime()) await cancel([widgetId]).catch(() => undefined);
      const state = await configurePomodoro(widgetId, workMinutes, breakMinutes);
      setPomodoros((current) => ({ ...current, [widgetId]: state }));
      setWidgetError("");
    } catch (reason) {
      setWidgetError(String(reason));
    }
  }

  return { widgets, pomodoros, widgetError, addWidget, saveWidget, duplicateWidget, removeWidget, moveWidget, controlPomodoro, savePomodoroDurations };
}
