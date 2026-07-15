import { useEffect } from "react";
import type { ThemePreference } from "./types";

const darkThemeQuery = "(prefers-color-scheme: dark)";

export function useTheme(preference: ThemePreference) {
  useEffect(() => {
    const media = window.matchMedia(darkThemeQuery);

    const applyTheme = () => {
      const resolved = preference === "system"
        ? (media.matches ? "dark" : "light")
        : preference;
      document.documentElement.dataset.theme = resolved;
      document.documentElement.dataset.themePreference = preference;
      document.documentElement.style.colorScheme = resolved;
    };

    applyTheme();
    if (preference !== "system") return;

    media.addEventListener("change", applyTheme);
    return () => media.removeEventListener("change", applyTheme);
  }, [preference]);
}
