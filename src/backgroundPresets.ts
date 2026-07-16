import type { BackgroundPreset } from "./types";

export const BACKGROUND_PRESETS: BackgroundPreset[] = [
  "foldedHorizon",
  "midnight",
  "graphite",
  "ember",
  "porcelain",
  "arctic",
  "linen",
  "morningMist",
];

export type BackgroundTone = "dark" | "light";

export function backgroundPresetTone(preset: BackgroundPreset): BackgroundTone {
  return (["porcelain", "arctic", "linen", "morningMist"] as BackgroundPreset[]).includes(preset)
    ? "light"
    : "dark";
}
