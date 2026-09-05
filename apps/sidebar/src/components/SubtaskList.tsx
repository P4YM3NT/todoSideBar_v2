import { useEffect, useState } from "react";
import { Maximize2, Plus, Trash2 } from "lucide-react";
import {
  type Task,
  createTask,
  deleteTask,
  listSubtasks,
  setTaskStatus,
} from "@/lib/tasks";
import { openDetail } from "@/lib/detail";
import { ICON_ACTION } from "@/lib/styles";
import { Button } from "@/components/ui/button";
import { DeleteTaskDialog } from "@/components/DeleteTaskDialog";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

/**
 * Unteraufgaben sind vollwertige Aufgaben und werden deshalb wie in der Sidebar
 * als Karten gezeigt - nur ohne Termin und Prioritaet, die haben im Kontext der
 * Hauptaufgabe keinen Platz. Ueber das Symbol oeffnet man sie als eigene Aufgabe
 * mit allen Feldern.
 */
export function SubtaskList({ parentId }: { parentId: string }) {
  const [items, setItems] = useState<Task[]>([]);
  const [draft, setDraft] = useState("");
  /** Die Unteraufgabe, deren Loeschen gerade bestaetigt werden soll. */
  const [confirming, setConfirming] = useState<Task | null>(null);

  const reload = () => void listSubtasks(parentId).then(setItems);
  useEffect(reload, [parentId]);

  const add = async () => {
    const title = draft.trim();
    if (!title) return;
    setDraft("");
    await createTask({ title, parentId });
    reload();
  };

  const doneCount = items.filter((t) => t.status === "done").length;

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-muted-foreground">
          Unteraufgaben
        </span>
        {items.length > 0 && (
          <span className="text-xs text-muted-foreground">
            {doneCount} / {items.length}
          </span>
        )}
      </div>

      {items.map((task) => {
        const done = task.status === "done";
          // Siehe TaskRow: einzeilige Karten muessen mittig sitzen.
          const multiline = Boolean(task.description);
        return (
          <div
            key={task.id}
            className={cn(
              "group flex cursor-pointer gap-3 rounded-md border border-border",
              "bg-background/40 px-3 py-2 transition-colors hover:bg-accent/50",
              multiline ? "items-start" : "items-center",
            )}
          >
            <Checkbox
              checked={done}
              onCheckedChange={async () => {
                await setTaskStatus(task.id, !done);
                reload();
              }}
              aria-label={done ? "Als offen markieren" : "Als erledigt markieren"}
              className={cn(
                multiline && "mt-0.5",
                "shrink-0 data-checked:border-(--checkbox-accent) data-checked:bg-(--checkbox-accent) data-checked:text-(--checkbox-accent-fg)",
              )}
            />

            <button
              type="button"
              onClick={() => void openDetail(task.id)}
              className="min-w-0 flex-1 space-y-1 rounded-md text-left outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
            >
              <span
                className={cn(
                  "block truncate text-sm leading-snug",
                  done && "text-muted-foreground line-through",
                )}
              >
                {task.title}
              </span>
              {task.description && (
                <span className="block truncate text-xs text-muted-foreground">
                  {task.description}
                </span>
              )}
            </button>

            <div className="flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Als eigene Aufgabe öffnen"
                onClick={() => void openDetail(task.id)}
                className={ICON_ACTION}
              >
                <Maximize2 className="size-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label={`${task.title} löschen`}
                onClick={() => setConfirming(task)}
                className={ICON_ACTION}
              >
                <Trash2 className="size-4" />
              </Button>
            </div>
          </div>
        );
      })}

      <div className="flex items-center gap-2">
        <Plus className="size-3.5 shrink-0 text-muted-foreground" />
        <Input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              void add();
            }
          }}
          onBlur={() => void add()}
          placeholder="Unteraufgabe hinzufügen"
          className="h-7 rounded-md border-0 bg-transparent px-2 -mx-2 text-sm shadow-none focus-visible:ring-1 dark:bg-transparent"
        />
      </div>

      <DeleteTaskDialog
        task={confirming}
        onOpenChange={(open) => !open && setConfirming(null)}
        onConfirm={async (task) => {
          setConfirming(null);
          await deleteTask(task.id);
          reload();
        }}
      />
    </div>
  );
}
