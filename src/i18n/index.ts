import { useEffect, useMemo, useState } from "react";
import type { LanguagePreference } from "../types";
import { en, type TranslationKey } from "./locales/en";
import { tr } from "./locales/tr";

type SupportedLanguage = "tr" | "en";
type TranslationParams = Record<string, string | number>;

const resources: Record<SupportedLanguage, Record<TranslationKey, string>> = { en, tr };

export function useI18n(preference: LanguagePreference) {
  const [language, setLanguage] = useState<SupportedLanguage>(() => resolveLanguage(preference));

  useEffect(() => {
    const applyLanguage = () => setLanguage(resolveLanguage(preference));
    applyLanguage();
    if (preference !== "system") return;

    window.addEventListener("languagechange", applyLanguage);
    return () => window.removeEventListener("languagechange", applyLanguage);
  }, [preference]);

  useEffect(() => {
    document.documentElement.lang = language;
    document.documentElement.dir = "ltr";
  }, [language]);

  return useMemo(() => {
    const locale = language === "tr" ? "tr-TR" : "en-US";
    const t = (key: TranslationKey, params?: TranslationParams) => translate(language, key, params);
    return {
      language,
      locale,
      t,
      localizeError: (message: string) => localizeNativeError(message, t),
      formatDate: (date: Date, style: "short" | "long" = "short") =>
        new Intl.DateTimeFormat(locale, style === "long"
          ? { day: "numeric", month: "long", weekday: "long" }
          : { day: "numeric", month: "long" }).format(date),
    };
  }, [language]);
}

function localizeNativeError(
  message: string,
  t: (key: TranslationKey, params?: TranslationParams) => string,
) {
  const taskId = message.match(/^(\d+) numaralı görev bulunamadı\.$/);
  if (taskId) return t("error.taskNotFound", { id: taskId[1] });

  const exact: Record<string, TranslationKey> = {
    "Görev başlığı boş olamaz.": "error.taskTitleEmpty",
    "Görev başlığı 120 karakterden uzun olamaz.": "error.taskTitleLong",
    "Saydamlık değeri 40 ile 100 arasında olmalıdır.": "error.opacity",
    "Otomatik sakin mod süresi 1 ile 120 dakika arasında olmalıdır.": "error.autoCalm",
    "Veritabanı bağlantısına erişilemedi.": "error.databaseLocked",
    "Monitörleri okuyacak bir pencere bulunamadı.": "error.monitorWindow",
    "Wallpaper penceresi bulunamadı.": "error.wallpaperWindow",
    "Kullanılabilir monitör bulunamadı.": "error.noMonitor",
    "Yönetim penceresi bulunamadı.": "error.controlWindow",
    "Masaüstü katmanı şu anda yalnızca Windows'ta destekleniyor.": "error.desktopLayer",
  };
  if (exact[message]) return t(exact[message]);
  if (message.startsWith("Veritabanı hatası:")) return t("error.database");
  if (message.startsWith("Monitör işlemi başarısız:")) return t("error.monitorOperation");
  if (message.startsWith("Pencere işlemi başarısız:")) return t("error.windowOperation");
  return t("error.native");
}

function resolveLanguage(preference: LanguagePreference): SupportedLanguage {
  if (preference === "tr" || preference === "en") return preference;
  return navigator.language.toLocaleLowerCase().startsWith("tr") ? "tr" : "en";
}

function translate(
  language: SupportedLanguage,
  key: TranslationKey,
  params: TranslationParams = {},
) {
  const message = resources[language][key] ?? resources.en[key];
  return message.replace(/\{\{(\w+)\}\}/g, (_, name: string) => String(params[name] ?? `{{${name}}}`));
}
