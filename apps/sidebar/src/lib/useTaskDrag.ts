import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import type { Task } from "./tasks";

type Drag = {
  id: string;
  pointerId: number;
  startY: number;
  startX: number;
  startScrollTop: number;
  x: number;
  y: number;
  active: boolean;
  index: number | null;
  ids: string[];
};

type Preview = { id: string; offset: number; index: number | null };

/**
 * shadcn bietet kein Sortable-Primitive. Pointer-Ereignisse funktionieren auch
 * in Tauri, ohne dessen Datei-Drop zu verändern; der Griff nutzt den UI-Button.
 */
export function useTaskDrag(
  tasks: Task[],
  leaving: Set<string>,
  onReorder?: (id: string, index: number) => void,
) {
  const listRef = useRef<HTMLDivElement>(null);
  const drag = useRef<Drag | null>(null);
  const latest = useRef({ tasks, leaving, onReorder });
  latest.current = { tasks, leaving, onReorder };
  const [preview, setPreview] = useState<Preview | null>(null);
  const [announcement, setAnnouncement] = useState("");

  useEffect(() => {
    let frame: number | null = null;
    const viewport = () => listRef.current?.closest<HTMLElement>("[data-slot='scroll-area-viewport']");

    const cancel = () => {
      drag.current = null;
      setPreview(null);
      if (frame !== null) cancelAnimationFrame(frame);
      frame = null;
    };

    const update = () => {
      const d = drag.current;
      const list = listRef.current;
      const area = viewport();
      if (!d || !list || !area) return;
      const current = latest.current;
      if (
        !current.onReorder || current.leaving.size > 0 ||
        current.tasks.length !== d.ids.length ||
        current.tasks.some((task, i) => task.id !== d.ids[i])
      ) {
        cancel();
        return;
      }

      const bounds = area.getBoundingClientRect();
      const inside = d.x >= bounds.left && d.x <= bounds.right &&
        d.y >= bounds.top && d.y <= bounds.bottom;
      const rows = [...list.querySelectorAll<HTMLElement>("[data-task-id]")]
        .filter((row) => row.dataset.taskId !== d.id);
      const before = rows.findIndex((row) => {
        const rect = row.getBoundingClientRect();
        return d.y < rect.top + rect.height / 2;
      });
      d.index = inside ? (before === -1 ? rows.length : before) : null;
      setPreview({
        id: d.id,
        offset: d.y - d.startY + area.scrollTop - d.startScrollTop,
        index: d.index,
      });
    };

    const autoScroll = () => {
      frame = null;
      const d = drag.current;
      const area = viewport();
      if (!d?.active || !area) return;
      const bounds = area.getBoundingClientRect();
      if (d.x >= bounds.left && d.x <= bounds.right &&
          d.y >= bounds.top - 32 && d.y <= bounds.bottom + 32) {
        const edge = Math.min(32, bounds.height / 4);
        const speed = d.y < bounds.top + edge
          ? -Math.min(12, (bounds.top + edge - d.y) / 3)
          : d.y > bounds.bottom - edge
            ? Math.min(12, (d.y - bounds.bottom + edge) / 3)
            : 0;
        if (speed !== 0) {
          area.scrollTop += speed;
          update();
        }
      }
      if (drag.current?.active) frame = requestAnimationFrame(autoScroll);
    };

    const onMove = (event: globalThis.PointerEvent) => {
      const d = drag.current;
      if (!d || event.pointerId !== d.pointerId) return;
      d.x = event.clientX;
      d.y = event.clientY;
      if (!d.active) {
        if (Math.hypot(d.x - d.startX, d.y - d.startY) < 4) return;
        d.active = true;
        frame = requestAnimationFrame(autoScroll);
      }
      event.preventDefault();
      update();
    };

    const onUp = (event: globalThis.PointerEvent) => {
      const d = drag.current;
      if (!d || event.pointerId !== d.pointerId) return;
      d.x = event.clientX;
      d.y = event.clientY;
      if (d.active) update();
      if (drag.current && d.active && d.index !== null) {
        const task = latest.current.tasks.find((item) => item.id === d.id);
        latest.current.onReorder?.(d.id, d.index);
        setAnnouncement(`${task?.title ?? "Aufgabe"}: Position ${d.index + 1} von ${d.ids.length}.`);
      }
      cancel();
    };

    const onKey = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape" && drag.current) {
        event.preventDefault();
        event.stopPropagation();
        cancel();
      }
    };

    const onCancel = (event: globalThis.PointerEvent) => {
      if (event.pointerId === drag.current?.pointerId) cancel();
    };

    window.addEventListener("pointermove", onMove, { passive: false });
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onCancel);
    window.addEventListener("blur", cancel);
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onCancel);
      window.removeEventListener("blur", cancel);
      window.removeEventListener("keydown", onKey, true);
      if (frame !== null) cancelAnimationFrame(frame);
      drag.current = null;
    };
  }, []);

  const start = (id: string, event: PointerEvent<HTMLButtonElement>) => {
    if (!onReorder || leaving.size > 0 || tasks.length < 2 ||
        event.button !== 0 || !event.isPrimary) return;
    event.preventDefault();
    event.currentTarget.focus();
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = {
      id,
      pointerId: event.pointerId,
      startY: event.clientY,
      startX: event.clientX,
      startScrollTop: listRef.current?.closest<HTMLElement>("[data-slot='scroll-area-viewport']")?.scrollTop ?? 0,
      x: event.clientX,
      y: event.clientY,
      active: false,
      index: null,
      ids: tasks.map((task) => task.id),
    };
  };

  const moveWithKeyboard = (id: string, event: KeyboardEvent<HTMLButtonElement>) => {
    if (!onReorder || leaving.size > 0 || drag.current) return;
    const from = tasks.findIndex((task) => task.id === id);
    const target = event.key === "ArrowUp" ? from - 1
      : event.key === "ArrowDown" ? from + 1
        : event.key === "Home" ? 0
          : event.key === "End" ? tasks.length - 1 : null;
    if (target === null) return;
    event.preventDefault();
    if (from < 0 || target < 0 || target >= tasks.length || target === from) return;
    onReorder(id, target);
    setAnnouncement(`${tasks[from].title}: Position ${target + 1} von ${tasks.length}.`);
    const handle = event.currentTarget;
    requestAnimationFrame(() => {
      if (handle.isConnected) handle.scrollIntoView({ block: "nearest" });
    });
  };

  const remaining = preview ? tasks.filter((task) => task.id !== preview.id) : [];
  return {
    listRef,
    preview,
    announcement,
    start,
    moveWithKeyboard,
    beforeId: preview?.index == null ? null : remaining[preview.index]?.id ?? null,
    atEnd: preview?.index != null && preview.index === remaining.length,
  };
}
