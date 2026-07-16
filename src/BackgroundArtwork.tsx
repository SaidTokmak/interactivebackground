import type { CSSProperties } from "react";
import type { BackgroundPreset } from "./types";

type Props = {
  preset: BackgroundPreset;
  compact?: boolean;
  custom?: boolean;
  style?: CSSProperties;
};

export function BackgroundArtwork({ preset, compact = false, custom = false, style }: Props) {
  return <span
    aria-hidden="true"
    className={`background-artwork preset-${preset} ${compact ? "background-swatch" : "desktop-background"} ${custom ? "custom-background" : ""}`}
    style={style}
  />;
}
