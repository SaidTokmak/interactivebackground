import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getSettings, isTauriRuntime, updateSettings } from "./taskApi";
import type { AppSettings } from "./types";

const defaults: AppSettings = {
  template: "focus",
  opacity: 82,
  editMode: false,
  monitorId: null,
};

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(defaults);
  const [settingsError, setSettingsError] = useState("");

  const refreshSettings = useCallback(async () => {
    try {
      setSettings(await getSettings());
      setSettingsError("");
    } catch (reason) {
      setSettingsError(String(reason));
    }
  }, []);

  useEffect(() => {
    void refreshSettings();
    if (!isTauriRuntime()) return;

    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    void listen("settings-changed", () => void refreshSettings()).then((stopListening) => {
      if (disposed) stopListening();
      else unlisten = stopListening;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refreshSettings]);

  async function saveSettings(next: AppSettings) {
    // UI'yi bekletmemek için önce yerel state güncellenir. Rust reddederse
    // veritabanındaki son geçerli değer tekrar yüklenir.
    setSettings(next);
    try {
      setSettings(await updateSettings(next));
      setSettingsError("");
    } catch (reason) {
      setSettingsError(String(reason));
      await refreshSettings();
    }
  }

  return { settings, settingsError, saveSettings };
}
