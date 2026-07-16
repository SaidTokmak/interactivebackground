import { useEffect, useState } from "react";
import { DEFAULT_GRID_SIZE } from "./widgetLayout";

const STORAGE_KEY = "interactivebackground.layout-grid-size";
const ALLOWED_GRID_SIZES = [0.005, 0.01] as const;

export function useLayoutGrid() {
  const [gridSize, setGridSize] = useState<number>(() => readGridSize());

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, String(gridSize));
    const synchronize = (event: StorageEvent) => {
      if (event.key === STORAGE_KEY) setGridSize(readGridSize());
    };
    window.addEventListener("storage", synchronize);
    return () => window.removeEventListener("storage", synchronize);
  }, [gridSize]);

  return { gridSize, setGridSize, gridSizes: ALLOWED_GRID_SIZES };
}

function readGridSize() {
  const stored = Number(localStorage.getItem(STORAGE_KEY));
  return ALLOWED_GRID_SIZES.find((size) => size === stored) ?? DEFAULT_GRID_SIZE;
}
