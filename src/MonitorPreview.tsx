import { useEffect, useRef, useState, type ReactNode } from "react";
import type { LayoutViewport } from "./widgetLayout";

type Props = {
  viewport?: LayoutViewport;
  children: ReactNode;
};

export function MonitorPreview({ viewport, children }: Props) {
  const stageRef = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(1);
  const width = viewport?.width ?? 1600;
  const height = viewport?.height ?? 900;

  useEffect(() => {
    const stage = stageRef.current;
    if (!stage) return;
    const updateScale = () => {
      const availableWidth = Math.max(1, stage.clientWidth - 36);
      const availableHeight = Math.min(window.innerHeight * 0.68, 760);
      setScale(Math.min(1, availableWidth / width, availableHeight / height));
    };
    updateScale();
    const observer = new ResizeObserver(updateScale);
    observer.observe(stage);
    window.addEventListener("resize", updateScale);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", updateScale);
    };
  }, [height, width]);

  return (
    <div className="monitor-preview-stage" ref={stageRef}>
      <div className="monitor-preview-frame" style={{ width: width * scale, height: height * scale }}>
        <div className="monitor-preview-content" style={{ width, height, transform: `scale(${scale})` }}>
          {children}
        </div>
      </div>
    </div>
  );
}
