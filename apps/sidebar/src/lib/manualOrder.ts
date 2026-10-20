/** Nur verschobene Aufgaben bekommen eine feste Position, je Ansicht. */
export type ManualPositions = Record<string, number>;

const PREFIX = "todosidebar.manualOrder.";

export function loadManualPositions(key: string): ManualPositions {
  try {
    const raw = localStorage.getItem(PREFIX + key);
    if (!raw) return {};
    const saved: unknown = JSON.parse(raw);
    if (!saved || typeof saved !== "object" || Array.isArray(saved)) return {};
    return Object.fromEntries(
      Object.entries(saved).filter(
        ([id, index]) => id.length > 0 && Number.isSafeInteger(index) && index >= 0,
      ),
    );
  } catch {
    return {};
  }
}

/** False bedeutet: Die Änderung gilt, konnte aber nicht gespeichert werden. */
export function saveManualPositions(key: string, positions: ManualPositions): boolean {
  try {
    if (Object.keys(positions).length === 0) localStorage.removeItem(PREFIX + key);
    else localStorage.setItem(PREFIX + key, JSON.stringify(positions));
    return true;
  } catch {
    return false;
  }
}

/** Freie Plätze werden weiterhin in der Reihenfolge der gewählten Sortierung gefüllt. */
export function applyManualPositions<T extends { id: string }>(
  tasks: T[],
  positions: ManualPositions,
): T[] {
  const pinned = tasks
    .filter((task) => Object.hasOwn(positions, task.id))
    .sort((a, b) => positions[a.id] - positions[b.id] || a.id.localeCompare(b.id));
  if (pinned.length === 0) return tasks;

  const ordered: (T | undefined)[] = Array.from({ length: tasks.length });
  let previous = -1;
  pinned.forEach((task, i) => {
    // Wenn ein Filter die Liste verkürzt, bleiben alle festen Positionen
    // innerhalb der Liste und behalten ihre relative Reihenfolge.
    const index = Math.max(
      previous + 1,
      Math.min(positions[task.id], tasks.length - pinned.length + i),
    );
    ordered[index] = task;
    previous = index;
  });

  const automatic = tasks.filter((task) => !Object.hasOwn(positions, task.id));
  let next = 0;
  return ordered.map((task) => task ?? automatic[next++]);
}

/** Zielposition bezieht sich auf die Liste nach Entfernen der gezogenen Aufgabe. */
export function moveTask<T extends { id: string }>(tasks: T[], id: string, index: number): T[] {
  const task = tasks.find((item) => item.id === id);
  if (!task) return tasks;
  const next = tasks.filter((item) => item.id !== id);
  next.splice(Math.max(0, Math.min(index, next.length)), 0, task);
  return next;
}

export function positionsAfterMove<T extends { id: string }>(
  ordered: T[],
  positions: ManualPositions,
  movedId: string,
): ManualPositions {
  const next = { ...positions };
  ordered.forEach((task, index) => {
    // Bereits verschobene Aufgaben nachziehen; alle anderen bleiben automatisch.
    if (task.id === movedId || Object.hasOwn(positions, task.id)) next[task.id] = index;
  });
  return next;
}
