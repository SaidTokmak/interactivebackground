import type { ClockWidgetSettings, LanguagePreference } from "./types";

export function defaultClockSettings(): ClockWidgetSettings {
  return { version: 1, style: "digital", hourFormat: "system", timeZone: null, showSeconds: true, showDate: true, showWeekday: true };
}

export function formatClock(date: Date, language: LanguagePreference, settings: ClockWidgetSettings) {
  return new Intl.DateTimeFormat(resolveLocale(language), {
    hour: "2-digit",
    minute: "2-digit",
    second: settings.showSeconds ? "2-digit" : undefined,
    hour12: settings.hourFormat === "system" ? undefined : settings.hourFormat === "hour12",
    timeZone: settings.timeZone ?? undefined,
  }).format(date);
}

export function formatClockDate(date: Date, language: LanguagePreference, settings: ClockWidgetSettings) {
  if (!settings.showDate && !settings.showWeekday) return "";
  return new Intl.DateTimeFormat(resolveLocale(language), {
    day: settings.showDate ? "2-digit" : undefined,
    month: settings.showDate ? "long" : undefined,
    year: settings.showDate ? "numeric" : undefined,
    weekday: settings.showWeekday ? "long" : undefined,
    timeZone: settings.timeZone ?? undefined,
  }).format(date);
}

export function clockZoneLabel(date: Date, language: LanguagePreference, timeZone: string | null) {
  const parts = new Intl.DateTimeFormat(resolveLocale(language), { timeZone: timeZone ?? undefined, timeZoneName: "short" }).formatToParts(date);
  return parts.find((part) => part.type === "timeZoneName")?.value ?? timeZone ?? "";
}

export function clockHandAngles(date: Date, timeZone: string | null) {
  const values = Object.fromEntries(new Intl.DateTimeFormat("en-US", {
    timeZone: timeZone ?? undefined,
    hour: "numeric",
    minute: "numeric",
    second: "numeric",
    hourCycle: "h23",
  }).formatToParts(date).map((part) => [part.type, part.value]));
  const hour = Number(values.hour ?? 0);
  const minute = Number(values.minute ?? 0);
  const second = Number(values.second ?? 0);
  return { hour: (hour % 12) * 30 + minute * .5, minute: minute * 6 + second * .1, second: second * 6 };
}

function resolveLocale(language: LanguagePreference) {
  if (language === "tr") return "tr-TR";
  if (language === "en") return "en-US";
  return typeof navigator === "undefined" ? "en-US" : navigator.language;
}
