import { invoke } from "@tauri-apps/api/core";
import type { SortSpec, TaskFilter } from "./filters";

export type Status = "open" | "done";

export type Task = {
  id: string;
  title: string;
  /** Einzeilig, erscheint in der Liste unter dem Titel. */
  description: string | null;
  /** Langer Freitext, nur im Detailfenster. */
  notes: string | null;
  projectId: string | null;
  /** Gesetzt, wenn dies eine Unteraufgabe ist. */
  parentId: string | null;
  status: Status;
  /** "Wann arbeite ich dran" - die Planung. */
  scheduledOn: string | null;
  /** Absolute instants; UI displays local time. */
  startsAt: string | null;
  endsAt: string | null;
  endsOn: string | null;
  /** "Bis wann muss es fertig sein" - die Deadline. */
  dueOn: string | null;
  priority: number;
  estimateMinutes: number | null;
  sortOrder: number;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type Project = {
  id: string;
  name: string;
  description: string | null;
  color: string | null;
  /** Name eines Lucide-Symbols; `PROJECT_ICONS` löst ihn auf. */
  icon: string | null;
  dueOn: string | null;
  sortOrder: number;
  /** Ob Unteraufgaben in der Projektliste mit auftauchen. */
  showSubtasks: boolean;
  archived: boolean;
};

export type ProjectFields = {
  name: string;
  description: string | null;
  color: string | null;
  icon: string | null;
  dueOn: string | null;
  showSubtasks: boolean;
};

/** Muss zur `TaskView`-Enum in `todo-core::models` passen. */
export type View =
  | { kind: "inbox" }
  | { kind: "today" }
  | { kind: "upcoming" }
  | { kind: "project"; value: string }
  | { kind: "done" }
  | { kind: "all" };

export type NewTask = {
  title: string;
  description?: string | null;
  notes?: string | null;
  projectId?: string | null;
  parentId?: string | null;
  scheduledOn?: string | null;
  startsAt?: string | null;
  endsAt?: string | null;
  endsOn?: string | null;
  dueOn?: string | null;
  priority?: number | null;
  estimateMinutes?: number | null;
};

export type TaskFields = {
  title: string;
  description: string | null;
  notes: string | null;
  projectId: string | null;
  scheduledOn: string | null;
  startsAt: string | null;
  endsAt: string | null;
  endsOn: string | null;
  dueOn: string | null;
  priority: number;
  estimateMinutes: number | null;
};

export const listTasks = (
  view: View,
  includeSubtasks = false,
  filter?: TaskFilter,
  sort?: SortSpec,
) => invoke<Task[]>("list_tasks", {
  view, includeSubtasks, filter: filter ?? null, sort: sort ?? null,
});

export const listSubtasks = (parentId: string) =>
  invoke<Task[]>("list_subtasks", { parentId });
export const createTask = (task: NewTask) => invoke<Task>("create_task", { task });

/** Freitextsuche über Titel, Beschreibung und Notizen - ansichtsübergreifend. */
export const searchTasks = (query: string, limit = 50) =>
  invoke<Task[]>("search_tasks", { query, limit });

/** Kopie samt Unteraufgaben, direkt unter dem Original. */
export const duplicateTask = (id: string) => invoke<Task>("duplicate_task", { id });
export const updateTask = (id: string, fields: TaskFields) =>
  invoke<Task>("update_task", { id, fields });
export const setTaskStatus = (id: string, done: boolean) =>
  invoke<Task>("set_task_status", { id, done });
export const deleteTask = (id: string) => invoke<void>("delete_task", { id });
export const reorderTask = (
  id: string,
  before: string | null,
  after: string | null,
) => invoke<void>("reorder_task", { id, before, after });

/** Farbe, die ein neu angelegtes Projekt bekommt - egal von wo. */
export const NEW_PROJECT_COLOR = "#3b82f6";

export const listProjects = (archived = false) =>
  invoke<Project[]>("list_projects", { archived });

/** Offene Aufgaben je Projekt-ID. */
export const projectOpenCounts = () =>
  invoke<Record<string, number>>("project_open_counts");

export const setProjectArchived = (id: string, archived: boolean) =>
  invoke<void>("set_project_archived", { id, archived });
export const createProject = (name: string, color?: string | null) =>
  invoke<Project>("create_project", { name, color: color ?? null });
export const updateProject = (id: string, fields: ProjectFields) =>
  invoke<void>("update_project", { id, fields });

export const deleteProject = (id: string) => invoke<void>("delete_project", { id });

export const viewKey = (v: View) =>
  v.kind === "project" ? `project:${v.value}` : v.kind;

/** Projekt, in das eine neue Aufgabe der aktuellen Ansicht gehört. */
export const defaultProjectFor = (v: View) =>
  v.kind === "project" ? v.value : null;
