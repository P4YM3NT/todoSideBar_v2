import {
  DEFAULT_SORT,
  EMPTY_FILTER,
  SORT_FIELDS,
  STATUS_LABEL,
  DUE_LABEL,
  type ViewSetting,
} from "./filters";

const PREFIX = "todosidebar.viewFilters.";

function fallback(): ViewSetting {
  return {
    filter: { ...EMPTY_FILTER, projects: [], priorities: [] },
    sort: { ...DEFAULT_SORT },
    presetId: null,
  };
}

/** Jede Ansicht merkt ihren eigenen Stand, wie die zuletzt gewählte Ansicht. */
export function loadViewSetting(key: string): ViewSetting {
  try {
    const raw = localStorage.getItem(PREFIX + key);
    if (!raw) return fallback();
    const saved = JSON.parse(raw) as ViewSetting;
    const { filter, sort, presetId } = saved;
    if (
      !filter || !sort ||
      !Array.isArray(filter.projects) ||
      !filter.projects.every((id) => id === null || typeof id === "string") ||
      !Array.isArray(filter.priorities) ||
      !filter.priorities.every((id) => Number.isSafeInteger(id) && id >= 0) ||
      !Object.hasOwn(STATUS_LABEL, filter.status) ||
      !Object.hasOwn(DUE_LABEL, filter.due) ||
      !SORT_FIELDS.includes(sort.field) ||
      (sort.direction !== "asc" && sort.direction !== "desc") ||
      (presetId !== null && (!Number.isSafeInteger(presetId) || presetId < 1))
    ) return fallback();
    return saved;
  } catch {
    return fallback();
  }
}

export function saveViewSetting(key: string, setting: ViewSetting): void {
  try {
    localStorage.setItem(PREFIX + key, JSON.stringify(setting));
  } catch {
    // Gesperrter oder voller Speicher darf die Ansicht nicht blockieren.
  }
}
