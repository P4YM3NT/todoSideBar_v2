import type { Priority } from "@/lib/ipc";
import type { Task } from "@/lib/tasks";
import type { ManualPositions } from "@/lib/manualOrder";
import { useTaskDrag } from "@/lib/useTaskDrag";
import { TaskRow } from "@/components/TaskRow";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";

type Props = {
  tasks: Task[];
  priorities: Priority[];
  leaving: Set<string>;
  loading: boolean;
  error: string | null;
  emptyHint: string;
  /** Ob ein Filter aktiv ist - der Leerzustand muss das benennen. */
  filtered: boolean;
  onResetFilter: () => void;
  onToggle: (task: Task) => void;
  onDuplicate: (id: string) => void;
  onDelete: (id: string) => void;
  onOpen: (task: Task) => void;
  onReorder?: (id: string, index: number) => void;
  manualPositions?: ManualPositions;
  onResetPosition?: (id: string) => void;
};

export function TaskList({
  tasks,
  priorities,
  leaving,
  loading,
  error,
  emptyHint,
  filtered,
  onResetFilter,
  onToggle,
  onDuplicate,
  onDelete,
  onOpen,
  onReorder,
  manualPositions = {},
  onResetPosition,
}: Props) {
  const drag = useTaskDrag(tasks, leaving, onReorder);
  // Der Rahmen ist in jedem Zustand da und immer gleich hoch, damit das
  // Eingabefeld darunter nicht springt. `min-h-0` erlaubt das Schrumpfen unter
  // die Inhaltshoehe - ohne das waechst die Liste ueber die Sidebar hinaus und
  // schiebt das Eingabefeld aus dem Fenster.
  const frame = (children: React.ReactNode) => (
    <div className="min-h-0 flex-1">{children}</div>
  );

  if (error) {
    return frame(
      <p className="px-2 py-1.5 text-xs text-destructive">{error}</p>,
    );
  }

  // Beim Laden bewusst nichts anzeigen: die Datenbank ist lokal und antwortet
  // in Millisekunden - ein Skeleton würde nur flackern.
  if (loading) return frame(null);

  if (tasks.length === 0) {
    // Ein zu enger Filter sieht sonst wie Datenverlust aus: die Liste ist leer
    // und nichts sagt einem, warum.
    return frame(
      filtered ? (
        <div className="space-y-2 px-2 py-1.5">
          <p className="text-xs text-muted-foreground">
            Keine Aufgabe passt zum Filter.
          </p>
          <Button variant="outline" size="sm" onClick={onResetFilter} className="w-full">
            Filter zurücksetzen
          </Button>
        </div>
      ) : (
        <p className="px-2 py-1.5 text-xs text-muted-foreground">{emptyHint}</p>
      ),
    );
  }

  return frame(
    <ScrollArea className="-mx-1 h-full px-1">
      <div ref={drag.listRef} className="relative space-y-2">
        {tasks.map((task) => (
          <div key={task.id} data-task-id={task.id} className="relative">
            {drag.beforeId === task.id && (
              <div aria-hidden className="pointer-events-none absolute -top-1 right-0 left-0 z-30 border-t-2 border-(--checkbox-accent)" />
            )}
            <TaskRow
              task={task}
              priorities={priorities}
              leaving={leaving.has(task.id)}
              onToggle={() => onToggle(task)}
              onDuplicate={() => onDuplicate(task.id)}
              onDelete={() => onDelete(task.id)}
              onOpen={() => onOpen(task)}
              onDragStart={onReorder ? (event) => drag.start(task.id, event) : undefined}
              onReorderKeyDown={(event) => drag.moveWithKeyboard(task.id, event)}
              onResetPosition={Object.hasOwn(manualPositions, task.id) && onResetPosition
                ? () => onResetPosition(task.id) : undefined}
              dragOffset={drag.preview?.id === task.id ? drag.preview.offset : undefined}
              reorderDisabled={tasks.length < 2 || leaving.size > 0}
            />
          </div>
        ))}
        {drag.atEnd && (
          <div aria-hidden className="pointer-events-none absolute right-0 -bottom-1 left-0 z-30 border-t-2 border-(--checkbox-accent)" />
        )}
        <p role="status" aria-live="polite" aria-atomic="true" className="sr-only">{drag.announcement}</p>
      </div>
    </ScrollArea>,
  );
}
