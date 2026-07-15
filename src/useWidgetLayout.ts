import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getWidgetLayout, isTauriRuntime, resetWidgetLayout, updateWidgetLayout } from "./taskApi";
import type { WallpaperTemplate, WidgetLayout } from "./types";

function defaults(monitorId: string | null, template: WallpaperTemplate): WidgetLayout {
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

export function useWidgetLayout(monitorId: string | null, template: WallpaperTemplate) {
  const [layout, setLayout] = useState<WidgetLayout>(() => defaults(monitorId, template));
  const [layoutError, setLayoutError] = useState("");

  const refreshLayout = useCallback(async () => {
    try {
      setLayout(await getWidgetLayout(monitorId, template));
      setLayoutError("");
    } catch (reason) {
      setLayoutError(String(reason));
    }
  }, [monitorId, template]);

  useEffect(() => {
    setLayout(defaults(monitorId, template));
    void refreshLayout();
    if (!isTauriRuntime()) return;

    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    void listen("widget-layout-changed", () => void refreshLayout()).then((stopListening) => {
      if (disposed) stopListening();
      else unlisten = stopListening;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [monitorId, template, refreshLayout]);

  async function saveLayout(next: WidgetLayout) {
    setLayout(next);
    try {
      setLayout(await updateWidgetLayout(next));
      setLayoutError("");
    } catch (reason) {
      setLayoutError(String(reason));
      await refreshLayout();
    }
  }

  async function restoreLayout() {
    try {
      setLayout(await resetWidgetLayout(monitorId, template));
      setLayoutError("");
    } catch (reason) {
      setLayoutError(String(reason));
    }
  }

  return { layout, layoutError, saveLayout, restoreLayout };
}
