import { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import type { PomodoroCompletion, PomodoroPreferences } from "./types";
import { getPomodoroPreferences, isTauriRuntime, updatePomodoroPreferences } from "./taskApi";
import { playPomodoroChime } from "./pomodoroSound";

export type NotificationPermissionState = "unknown" | "granted" | "denied";

const defaults: PomodoroPreferences = {
  notificationsEnabled: true,
  soundEnabled: true,
  soundVolume: 70,
};

export function usePomodoroAlerts() {
  const [preferences, setPreferences] = useState(defaults);
  const preferencesRef = useRef(preferences);
  const [permission, setPermission] = useState<NotificationPermissionState>(isTauriRuntime() ? "unknown" : "granted");
  const [alertError, setAlertError] = useState("");

  useEffect(() => { preferencesRef.current = preferences; }, [preferences]);

  const refresh = useCallback(async () => {
    try {
      setPreferences(await getPomodoroPreferences());
      if (isTauriRuntime()) setPermission(await isPermissionGranted() ? "granted" : "denied");
      setAlertError("");
    } catch (reason) {
      setAlertError(String(reason));
    }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);
  useEffect(() => {
    if (!isTauriRuntime()) return;
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];
    void Promise.all([
      listen("pomodoro-preferences-changed", () => void refresh()),
      listen<PomodoroCompletion>("pomodoro-completed", () => {
        const current = preferencesRef.current;
        if (current.soundEnabled) void playPomodoroChime(current.soundVolume).catch((reason) => setAlertError(String(reason)));
      }),
    ]).then((stops) => disposed ? stops.forEach((stop) => stop()) : unlisteners.push(...stops));
    return () => { disposed = true; unlisteners.forEach((stop) => stop()); };
  }, [refresh]);

  async function savePreferences(next: PomodoroPreferences) {
    setPreferences(next);
    preferencesRef.current = next;
    try {
      setPreferences(await updatePomodoroPreferences(next));
      setAlertError("");
    } catch (reason) {
      setAlertError(String(reason));
      await refresh();
    }
  }

  async function requestNotificationPermission() {
    if (!isTauriRuntime()) {
      setPermission("granted");
      return;
    }
    try {
      const result = await requestPermission();
      setPermission(result === "granted" ? "granted" : "denied");
      setAlertError("");
    } catch (reason) {
      setPermission("denied");
      setAlertError(String(reason));
    }
  }

  async function testSound() {
    try {
      await playPomodoroChime(preferences.soundVolume);
      setAlertError("");
    } catch (reason) {
      setAlertError(String(reason));
    }
  }

  return { preferences, permission, alertError, savePreferences, requestNotificationPermission, testSound };
}
