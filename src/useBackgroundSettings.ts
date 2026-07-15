import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getBackgroundSettings, isTauriRuntime, updateBackgroundSettings } from "./taskApi";
import type { BackgroundSettings } from "./types";

function defaults(monitorId: string | null): BackgroundSettings {
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

export function useBackgroundSettings(monitorId: string | null) {
  const [background, setBackground] = useState<BackgroundSettings>(() => defaults(monitorId));
  const [backgroundError, setBackgroundError] = useState("");

  const refreshBackground = useCallback(async () => {
    try {
      setBackground(await getBackgroundSettings(monitorId));
      setBackgroundError("");
    } catch (reason) {
      setBackgroundError(String(reason));
    }
  }, [monitorId]);

  useEffect(() => {
    setBackground(defaults(monitorId));
    void refreshBackground();
    if (!isTauriRuntime()) return;

    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    void listen("background-settings-changed", () => void refreshBackground()).then((stopListening) => {
      if (disposed) stopListening();
      else unlisten = stopListening;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [monitorId, refreshBackground]);

  async function saveBackground(next: BackgroundSettings) {
    setBackground(next);
    try {
      setBackground(await updateBackgroundSettings(next));
      setBackgroundError("");
    } catch (reason) {
      setBackgroundError(String(reason));
      await refreshBackground();
    }
  }

  return { background, backgroundError, saveBackground, refreshBackground };
}
