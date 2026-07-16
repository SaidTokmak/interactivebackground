import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauriRuntime, listWidgetPackages, setWidgetPackageInstalled } from "./taskApi";
import type { WidgetKind, WidgetPackage } from "./types";

export function useWidgetStore() {
  const [packages, setPackages] = useState<WidgetPackage[]>([]);
  const [storeError, setStoreError] = useState("");

  const refreshPackages = useCallback(async () => {
    try {
      setPackages(await listWidgetPackages());
      setStoreError("");
    } catch (reason) {
      setStoreError(String(reason));
    }
  }, []);

  useEffect(() => {
    void refreshPackages();
    if (!isTauriRuntime()) return;
    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    void listen("widget-packages-changed", () => void refreshPackages()).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refreshPackages]);

  async function setInstalled(kind: WidgetKind, installed: boolean) {
    try {
      const updated = await setWidgetPackageInstalled(kind, installed);
      setPackages((current) => current.map((item) => item.kind === kind ? updated : item));
      setStoreError("");
    } catch (reason) {
      setStoreError(String(reason));
    }
  }

  return { packages, storeError, setInstalled };
}
