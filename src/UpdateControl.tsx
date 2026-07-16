import { useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { isTauriRuntime } from "./taskApi";
import type { TranslationKey } from "./i18n/locales/en";

type UpdateState = "idle" | "checking" | "available" | "downloading" | "current" | "error";

interface UpdateControlProps {
  t: (key: TranslationKey, params?: Record<string, string | number>) => string;
}

export function UpdateControl({ t }: UpdateControlProps) {
  const [state, setState] = useState<UpdateState>("idle");
  const [update, setUpdate] = useState<Update | null>(null);
  const [progress, setProgress] = useState(0);

  const configured = import.meta.env.VITE_UPDATER_ENABLED === "true";
  if (isTauriRuntime() && !configured) return null;

  async function checkForUpdate() {
    if (!isTauriRuntime()) {
      setState("current");
      return;
    }
    setState("checking");
    try {
      const available = await check({ timeout: 15_000 });
      if (!available) {
        setState("current");
        return;
      }
      setUpdate(available);
      setState("available");
    } catch (reason) {
      console.error("Update check failed", reason);
      setState("error");
    }
  }

  async function installUpdate() {
    if (!update) return;
    setState("downloading");
    setProgress(0);
    let downloaded = 0;
    let total = 0;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setProgress(total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0);
        }
        if (event.event === "Finished") setProgress(100);
      });
    } catch (reason) {
      console.error("Update install failed", reason);
      setState("error");
    }
  }

  if (state === "available") {
    return (
      <button className="update-button is-available" onClick={() => void installUpdate()}>
        {t("update.install", { version: update?.version ?? "" })}
      </button>
    );
  }

  if (state === "downloading") {
    return <button className="update-button" disabled>{t("update.downloading", { progress })}</button>;
  }

  return (
    <button
      className={`update-button ${state === "error" ? "has-error" : ""}`}
      disabled={state === "checking"}
      onClick={() => void checkForUpdate()}
      title={state === "error" ? t("update.errorHelp") : undefined}
    >
      {state === "checking" ? t("update.checking") : state === "current" ? t("update.current") : state === "error" ? t("update.retry") : t("update.check")}
    </button>
  );
}
