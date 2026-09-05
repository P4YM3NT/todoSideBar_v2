import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type DetailTarget = {
  /** `null` = neue Aufgabe anlegen. */
  taskId: string | null;
  prefillTitle: string | null;
  prefillProjectId: string | null;
};

export const openDetail = (
  taskId: string | null,
  prefill?: { title?: string | null; projectId?: string | null },
) =>
  invoke<void>("open_detail", {
    taskId,
    prefillTitle: prefill?.title ?? null,
    prefillProjectId: prefill?.projectId ?? null,
  });

/** Versteckt das Fenster; beim nächsten Öffnen ist es sofort wieder da. */
export const closeDetail = () => invoke<void>("close_detail");

/**
 * Blendet das Fenster ein. Es entsteht unsichtbar und wartet darauf - sonst
 * sähe man erst eine leere Fläche, dann den Ladekreis, dann die Felder
 * einzeln nachrücken.
 */
export const showDetail = () => invoke<void>("detail_ready");

/** Auf die Arbeitsfläche vergrößern und zurück. Liefert den neuen Zustand. */
export const toggleDetailMaximized = () =>
  invoke<boolean>("toggle_detail_maximized");

/** Verhindert den grauen Blitz beim Vergrößern des Fensters. */
export const setDetailBackground = (r: number, g: number, b: number) =>
  invoke<void>("set_detail_background", { r, g, b });

/** Beim Start holt sich das Fenster sein Ziel selbst - Events koennen es verpassen. */
export const getDetailTarget = () => invoke<string | null>("detail_target");

export const onDetailTarget = (cb: (t: DetailTarget) => void) =>
  listen<DetailTarget>("detail://target", (e) => cb(e.payload));

/** Sagt der Sidebar, dass sie neu laden soll. */
export const notifyTasksChanged = () => invoke<void>("notify_tasks_changed");

export const onTasksChanged = (cb: () => void) =>
  listen("tasks://changed", () => cb());
