import { useCallback, useEffect, useState } from "react";
import { type Task, searchTasks } from "./tasks";
import { onTasksChanged } from "./detail";

/**
 * Tippen soll nicht jede Taste an die Datenbank schicken. Kurz gehalten - die
 * Abfrage läuft lokal in unter einer Millisekunde, längeres Warten würde sich
 * nur zäh anfühlen.
 */
const DEBOUNCE_MS = 120;

/**
 * Sucht ansichtsübergreifend. Anders als `useTasks` hält der Hook keine
 * Projekte und Zähler - die Trefferliste ist eine Momentaufnahme, kein Zustand,
 * in dem man arbeitet.
 */
export function useSearch(query: string) {
  const [results, setResults] = useState<Task[]>([]);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async () => {
    const needle = query.trim();
    if (!needle) {
      setResults([]);
      setError(null);
      return;
    }
    try {
      setResults(await searchTasks(needle));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [query]);

  useEffect(() => {
    const id = setTimeout(() => void run(), DEBOUNCE_MS);
    return () => clearTimeout(id);
  }, [run]);

  // Das Detailfenster schreibt in dieselbe Datenbank - ohne dieses Signal
  // zeigt die Trefferliste einen überholten Stand.
  useEffect(() => {
    const sub = onTasksChanged(() => void run());
    return () => void sub.then((un) => un());
  }, [run]);

  return { results, error, refresh: run };
}
