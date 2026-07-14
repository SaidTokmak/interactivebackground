import type { CSSProperties } from "react";
import type { Task, TaskStatus, WallpaperTemplate } from "./types";
import appIcon from "./assets/interactivebackground-icon.png";

type Props = {
  tasks: Task[];
  template: WallpaperTemplate;
  editMode: boolean;
  opacity: number;
  actual?: boolean;
  onToggle: (id: number) => void;
  onMove: (id: number, status: TaskStatus) => void;
};

const columns = [
  { label: "Yapılacak", status: "todo" as const },
  { label: "Devam ediyor", status: "inProgress" as const },
  { label: "Bitti", status: "done" as const },
];

export function WallpaperSurface({ tasks, template, editMode, opacity, actual = false, onToggle, onMove }: Props) {
  const completed = tasks.filter((task) => task.status === "done").length;
  const progress = tasks.length === 0 ? 0 : Math.round((completed / tasks.length) * 100);
  const nextTask = tasks.find((task) => task.status !== "done");

  return (
    <div className={`desktop-preview ${actual ? "actual-surface" : ""}`}>
      <div className="desktop-topline">
        <span className="desktop-brand"><img src={appIcon} alt="" aria-hidden="true" />interactivebackground</span>
        <span className="desktop-mode">⌁ {editMode ? "Düzenleme modu" : "Sakin mod"}</span>
      </div>
      <div className="desktop-icon"><span>▱</span>Projeler</div>
      <div className="desktop-icon second"><span>♲</span>Çöp Kutusu</div>

      <section className={`wallpaper-widget ${actual ? "actual-widget" : ""} ${editMode ? "editing" : ""}`} style={{ backgroundColor: `color-mix(in srgb, var(--widget) ${opacity}%, transparent)` }}>
        <div className="widget-header">
          <div><h3>{template === "focus" ? "Bugünün odağı" : "Ürün panosu"}</h3><span>14 Temmuz · Salı</span></div>
          <div className="progress-circle" style={{ "--progress": `${progress * 3.6}deg` } as CSSProperties}>
            <b>{completed}/{tasks.length}</b>
          </div>
        </div>

        {template === "focus" ? (
          <>
            <div className="widget-tasks">
              {tasks.slice(0, 6).map((task) => (
                <label className={task.status === "done" ? "done" : ""} key={task.id}>
                  <input type="checkbox" checked={task.status === "done"} onChange={() => onToggle(task.id)} />
                  <span>{task.title}</span>
                  <time>{task.scheduledFor}</time>
                </label>
              ))}
            </div>
            <div className="focus-action">
              <span>Sıradaki: {nextTask?.title ?? "Hepsi tamamlandı"}</span>
              <button>▶ Odaklan</button>
            </div>
          </>
        ) : (
          <div className="kanban-board">
            {columns.map((column) => (
              <div className="kanban-column" key={column.status}>
                <span>{column.label}</span>
                {tasks.filter((task) => task.status === column.status).map((task) => (
                  <button className="kanban-card" key={task.id} onClick={() => onMove(task.id, nextStatus(column.status))}>
                    {task.title}
                  </button>
                ))}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function nextStatus(status: TaskStatus): TaskStatus {
  if (status === "todo") return "inProgress";
  if (status === "inProgress") return "done";
  return "todo";
}
