import { useCallback, useEffect, useState } from "react";
import {
  type NewTask,
  type Project,
  type Task,
  type View,
  NEW_PROJECT_COLOR,
  createProject,
  createTask,
  deleteTask,
  duplicateTask,
  listProjects,
  listTasks,
  projectOpenCounts,
  setTaskStatus,
  viewKey,
} from "./tasks";
import { notifyTasksChanged, onTasksChanged } from "./detail";
import { DEFAULT_SORT, EMPTY_FILTER, type SortSpec, type TaskFilter } from "./filters";

/** Wie lange der gesetzte Haken stehen bleibt, bevor die Zeile geht. */
const LINGER_MS = 320;
/** Dauer des Hinausgleitens - muss zur Transition in `TaskRow` passen. */
const LEAVE_MS = 380;

/**
 * Lädt Aufgaben und Projekte für die aktuelle Ansicht.
 *
 * Bewusst ohne Server-State-Bibliothek: die Daten liegen lokal in SQLite,
 * es gibt kein Netzwerk, kein Caching-Problem und keine Revalidierung.
 */
export function useTasks(
  view: View,
  showSubtasks: boolean,
  filter: TaskFilter = EMPTY_FILTER,
  sort: SortSpec = DEFAULT_SORT,
) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [counts, setCounts] = useState<Record<string, number>>({});
  const [error, setError] = useState<string | null>(null);
  /** Zeilen, die gerade hinausgleiten. */
  const [leaving, setLeaving] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);

  const key = viewKey(view);

  const reload = useCallback(async () => {
    try {
      const projectList = await listProjects();
      // Projekte duerfen die globale Einstellung ueberschreiben.
      const project =
        view.kind === "project"
          ? projectList.find((p) => p.id === view.value)
          : undefined;
      const include = project ? project.showSubtasks : showSubtasks;
      const [t, c] = await Promise.all([
        listTasks(view, include, filter, sort),
        projectOpenCounts(),
      ]);
      const p = projectList;
      setCounts(c);
      setTasks(t);
      setProjects(p);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
    // `key` statt `view`: das Objekt ist bei jedem Render neu, der Schlüssel nicht.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, showSubtasks, filter, sort]);

  useEffect(() => {
    setLoading(true);
    void reload();
  }, [reload]);

  // Das Detailfenster schreibt in dieselbe Datenbank - ohne dieses Signal
  // zeigt die Sidebar bis zum naechsten Ansichtswechsel den alten Stand.
  useEffect(() => {
    const sub = onTasksChanged(() => void reload());
    return () => void sub.then((un) => un());
  }, [reload]);

  const add = useCallback(
    async (task: NewTask) => {
      await createTask(task);
      await reload();
    },
    [reload],
  );

  /**
   * Abhaken laeuft in drei Schritten, damit man die Bestaetigung sieht:
   * 1. Haken sofort setzen (sonst blinkt die Checkbox bis zum Neuladen)
   * 2. kurz stehen lassen, dann die Zeile hinausgleiten lassen
   * 3. erst danach speichern und neu laden
   *
   * In der Ansicht "Alle" bleibt die Aufgabe sichtbar - dort wird nicht animiert.
   */
  const toggle = useCallback(
    async (task: Task) => {
      const done = task.status === "open";
      const status = filter.status === "inherit"
        ? (view.kind === "all" ? "all" : view.kind === "done" ? "done" : "open")
        : filter.status;
      const leaves = status !== "all";

      setTasks((prev) =>
        prev.map((t) =>
          t.id === task.id ? { ...t, status: done ? "done" : "open" } : t,
        ),
      );

      if (leaves) {
        await new Promise((r) => setTimeout(r, LINGER_MS));
        setLeaving((prev) => new Set(prev).add(task.id));
        await new Promise((r) => setTimeout(r, LEAVE_MS));
      }

      try {
        await setTaskStatus(task.id, done);
      } finally {
        await reload();
        setLeaving((prev) => {
          const next = new Set(prev);
          next.delete(task.id);
          return next;
        });
      }
    },
    [reload, view.kind, filter.status],
  );

  /**
   * Legt ein Projekt an. Das Detailfenster hat eine eigene Projektauswahl -
   * ohne das Signal kennt sie das neue Projekt bis zum Neustart nicht.
   */
  const addProject = useCallback(
    async (name: string) => {
      await createProject(name, NEW_PROJECT_COLOR);
      await notifyTasksChanged();
      await reload();
    },
    [reload],
  );

  /** Kopie samt Unteraufgaben, direkt unter dem Original. */
  const duplicate = useCallback(
    async (id: string) => {
      await duplicateTask(id);
      await reload();
    },
    [reload],
  );

  const remove = useCallback(
    async (id: string) => {
      setTasks((prev) => prev.filter((t) => t.id !== id));
      try {
        await deleteTask(id);
      } finally {
        await reload();
      }
    },
    [reload],
  );

  return {
    tasks,
    projects,
    counts,
    loading,
    error,
    leaving,
    reload,
    add,
    addProject,
    toggle,
    duplicate,
    remove,
  };
}
