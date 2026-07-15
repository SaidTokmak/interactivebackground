import type { CSSProperties } from "react";
import type { LanguagePreference, Task, TaskStatus, WallpaperTemplate } from "./types";
import appIcon from "./assets/interactivebackground-icon.png";
import { useI18n } from "./i18n";

type Props = {
  tasks: Task[];
  template: WallpaperTemplate;
  editMode: boolean;
  opacity: number;
  language: LanguagePreference;
  actual?: boolean;
  onToggle: (id: number) => void;
  onMove: (id: number, status: TaskStatus) => void;
};

export function WallpaperSurface({ tasks, template, editMode, opacity, language, actual = false, onToggle, onMove }: Props) {
  const { t, formatDate } = useI18n(language);
  const completed = tasks.filter((task) => task.status === "done").length;
  const progress = tasks.length === 0 ? 0 : Math.round((completed / tasks.length) * 100);
  const nextTask = tasks.find((task) => task.status !== "done");
  const columns = [
    { label: t("kanban.todo"), status: "todo" as const },
    { label: t("kanban.inProgress"), status: "inProgress" as const },
    { label: t("kanban.done"), status: "done" as const },
  ];

  return (
    <div className={`desktop-preview ${actual ? "actual-surface" : ""}`}>
      <div className="desktop-topline">
        <span className="desktop-brand"><img src={appIcon} alt="" aria-hidden="true" />interactivebackground</span>
        <span className="desktop-mode">⌁ {editMode ? t("wallpaper.mode.edit") : t("wallpaper.mode.calm")}</span>
      </div>
      <div className="desktop-icon"><span>▱</span>{t("desktop.projects")}</div>
      <div className="desktop-icon second"><span>♲</span>{t("desktop.trash")}</div>

      <section className={`wallpaper-widget ${actual ? "actual-widget" : ""} ${editMode ? "editing" : ""}`} style={{ backgroundColor: `color-mix(in srgb, var(--widget) ${opacity}%, transparent)` }}>
        <div className="widget-header">
          <div><h3>{template === "focus" ? t("widget.focusTitle") : t("widget.boardTitle")}</h3><span>{formatDate(new Date(), "long")}</span></div>
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
              <span>{nextTask ? t("widget.next", { title: nextTask.title }) : t("widget.allDone")}</span>
              <button>{t("widget.focusButton")}</button>
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
