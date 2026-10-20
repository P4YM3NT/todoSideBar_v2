import { useEffect, useState } from "react";
import type { Task } from "./tasks";
import {
  applyManualPositions,
  loadManualPositions,
  moveTask,
  positionsAfterMove,
  saveManualPositions,
  type ManualPositions,
} from "./manualOrder";

export function useManualOrder(key: string, tasks: Task[]) {
  const [saved, setSaved] = useState(() => ({ key, positions: loadManualPositions(key) }));
  const [error, setError] = useState<string | null>(null);
  const positions = saved.key === key ? saved.positions : loadManualPositions(key);
  const ordered = applyManualPositions(tasks, positions);

  useEffect(() => {
    setSaved({ key, positions: loadManualPositions(key) });
    setError(null);
  }, [key]);

  const change = (next: ManualPositions) => {
    setSaved({ key, positions: next });
    setError(saveManualPositions(key, next)
      ? null
      : "Reihenfolge konnte nicht gespeichert werden. Sie gilt bis zum Ansichtswechsel.");
  };

  const reorder = (id: string, index: number) => {
    if (ordered.findIndex((task) => task.id === id) === index) return;
    const next = moveTask(ordered, id, index);
    if (next === ordered) return;
    change(positionsAfterMove(next, positions, id));
  };

  const resetItem = (id: string) => {
    const next = { ...positions };
    delete next[id];
    change(next);
  };

  return {
    ordered,
    positions,
    error,
    reorder,
    resetItem,
    reset: () => change({}),
    hasManualOrder: Object.keys(positions).length > 0,
  };
}
