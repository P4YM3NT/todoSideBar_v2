/** Entspricht den Filter- und Sortier-Typen in todo_core::models. */
export type SortField =
  | "manual"
  | "dueOn"
  | "scheduledOn"
  | "priority"
  | "title"
  | "createdAt"
  | "completedAt";
export type SortSpec = { field: SortField; direction: "asc" | "desc" };
export type StatusFilter = "inherit" | "open" | "done" | "all";
export type DueFilter = "any" | "overdue" | "today" | "week" | "noDate";
export type TaskFilter = {
  projects: (string | null)[];
  priorities: number[];
  status: StatusFilter;
  due: DueFilter;
};
export type FilterPreset = {
  id: number;
  name: string;
  filter: TaskFilter;
  sort: SortSpec;
};
export type ViewSetting = {
  filter: TaskFilter;
  sort: SortSpec;
  presetId: number | null;
};

export const EMPTY_FILTER: TaskFilter = {
  projects: [],
  priorities: [],
  status: "inherit",
  due: "any",
};
export const DEFAULT_SORT: SortSpec = { field: "manual", direction: "asc" };
export const SORT_FIELDS: SortField[] = [
  "manual", "dueOn", "scheduledOn", "priority", "title", "createdAt", "completedAt",
];
export const SORT_LABEL: Record<SortField, string> = {
  manual: "Reihenfolge",
  dueOn: "Fälligkeit",
  scheduledOn: "Geplantes Datum",
  priority: "Priorität",
  title: "Titel",
  createdAt: "Erstellung",
  completedAt: "Abschluss",
};
export const STATUS_LABEL: Record<StatusFilter, string> = {
  inherit: "Status der Ansicht",
  open: "Offen",
  done: "Erledigt",
  all: "Alle Status",
};
export const DUE_LABEL: Record<DueFilter, string> = {
  any: "Alle Fälligkeiten",
  overdue: "Überfällig",
  today: "Heute fällig",
  week: "Nächste 7 Tage",
  noDate: "Ohne Fälligkeit",
};

/** Zählt eingeschränkte Filtergruppen, nicht einzelne Projekt-IDs. */
export const activeFilterCount = (filter: TaskFilter) =>
  Number(filter.projects.length > 0) +
  Number(filter.priorities.length > 0) +
  Number(filter.status !== "inherit") +
  Number(filter.due !== "any");

export const isSettingActive = (setting: ViewSetting) =>
  activeFilterCount(setting.filter) > 0 || setting.sort.field !== "manual";

export function toggleIn<T>(values: T[], value: T): T[] {
  return values.includes(value)
    ? values.filter((item) => item !== value)
    : [...values, value];
}

function sameSelection<T>(a: T[], b: T[]): boolean {
  const left = new Set(a);
  const right = new Set(b);
  return left.size === right.size && [...left].every((value) => right.has(value));
}

export const sameFilter = (a: TaskFilter, b: TaskFilter) =>
  a.status === b.status && a.due === b.due &&
  sameSelection(a.projects, b.projects) && sameSelection(a.priorities, b.priorities);

export const sameSort = (a: SortSpec, b: SortSpec) =>
  a.field === b.field && (a.field === "manual" || a.direction === b.direction);
