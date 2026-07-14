import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import * as taskApi from "./taskApi";
import type { Task, TaskStatus } from "./types";

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [error, setError] = useState("");

  const refresh = useCallback(async () => {
    try {
      setTasks(await taskApi.listTasks());
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }, []);

  useEffect(() => {
    void refresh();
    if (!taskApi.isTauriRuntime()) return;

    let disposed = false;
    let unlisten: UnlistenFn | undefined;

    // Rust her değişiklikten sonra bu olayı bütün pencerelere yayınlar.
    // Event yalnızca invalidation sinyalidir; güncel veri tekrar SQLite'tan okunur.
    void listen("tasks-changed", () => void refresh()).then((stopListening) => {
      if (disposed) stopListening();
      else unlisten = stopListening;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refresh]);

  async function addTask(title: string, scheduledFor: string | null) {
    try {
      const task = await taskApi.createTask(title, scheduledFor);
      setTasks((current) => [...current.filter((item) => item.id !== task.id), task]);
      setError("");
      return true;
    } catch (reason) {
      setError(String(reason));
      return false;
    }
  }

  async function toggleTask(id: number) {
    try {
      const updated = await taskApi.toggleTask(id);
      setTasks((current) => current.map((task) => (task.id === id ? updated : task)));
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function moveTask(id: number, status: TaskStatus) {
    try {
      const updated = await taskApi.moveTask(id, status);
      setTasks((current) => current.map((task) => (task.id === id ? updated : task)));
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function removeTask(id: number) {
    try {
      await taskApi.deleteTask(id);
      setTasks((current) => current.filter((task) => task.id !== id));
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  return { tasks, error, addTask, toggleTask, moveTask, removeTask };
}
