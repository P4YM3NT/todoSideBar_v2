import type { Priority } from "@/lib/ipc";
import type { Task } from "@/lib/tasks";
import { TaskRow } from "@/components/TaskRow";
import { ScrollArea } from "@/components/ui/scroll-area";

type Props = {
  tasks: Task[];
  priorities: Priority[];
  leaving: Set<string>;
  loading: boolean;
  error: string | null;
  emptyHint: string;
  onToggle: (task: Task) => void;
  onDuplicate: (id: string) => void;
  onDelete: (id: string) => void;
  onOpen: (task: Task) => void;
};

export function TaskList({
  tasks,
  priorities,
  leaving,
  loading,
  error,
  emptyHint,
  onToggle,
  onDuplicate,
  onDelete,
  onOpen,
}: Props) {
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
    return frame(
      <p className="px-2 py-1.5 text-xs text-muted-foreground">{emptyHint}</p>,
    );
  }

  return frame(
    <ScrollArea className="-mx-1 h-full px-1">
      <div className="space-y-2">
        {tasks.map((task) => (
          <TaskRow
            key={task.id}
            task={task}
            priorities={priorities}
            leaving={leaving.has(task.id)}
            onToggle={() => onToggle(task)}
            onDuplicate={() => onDuplicate(task.id)}
            onDelete={() => onDelete(task.id)}
            onOpen={() => onOpen(task)}
          />
        ))}
      </div>
    </ScrollArea>,
  );
}
