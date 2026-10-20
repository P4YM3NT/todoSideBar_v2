import { useEffect, useRef, useState } from "react";
import { Search, X } from "lucide-react";
import type { Priority } from "@/lib/ipc";
import {
  type Task,
  deleteTask,
  duplicateTask,
  setTaskStatus,
} from "@/lib/tasks";
import { openDetail } from "@/lib/detail";
import { ICON_ACTION } from "@/lib/styles";
import { useSearch } from "@/lib/useSearch";
import { TaskList } from "@/components/TaskList";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type Props = {
  priorities: Priority[];
  onClose: () => void;
  /** Zähler und Ansichtsliste der Sidebar nachziehen. */
  onChanged: () => void;
};

/**
 * Die Suche ist ein eigener Modus, kein Filter über der Ansicht: sie ersetzt
 * Kopfzeile und Liste. In einer 220 px schmalen Spalte ist für Feld, Navigation
 * und Treffer nebeneinander kein Platz - und wer sucht, sucht ohnehin quer
 * durch alle Ansichten.
 */
export function SearchView({ priorities, onClose, onChanged }: Props) {
  const [query, setQuery] = useState("");
  const { results, error, refresh } = useSearch(query);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Nach jeder Änderung beides nachziehen: die Trefferliste, damit der Haken
  // stehen bleibt, und die Sidebar dahinter, damit ihre Zähler stimmen.
  const after = async (action: Promise<unknown>) => {
    await action;
    await refresh();
    onChanged();
  };

  return (
    <div className="flex h-full flex-col gap-4">
      <header className="flex shrink-0 items-center gap-1">
        <div className="relative min-w-0 flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") onClose();
            }}
            placeholder="Aufgaben durchsuchen"
            className="h-7 pl-8 text-sm"
          />
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={onClose}
          aria-label="Suche schließen"
          className={ICON_ACTION}
        >
          <X className="size-4" />
        </Button>
      </header>

      {query.trim() === "" ? (
        <p className="px-2 py-1.5 text-xs text-muted-foreground">
          Sucht in Titel, Beschreibung und Notizen - über alle Ansichten hinweg.
        </p>
      ) : (
        <TaskList
          tasks={results}
          priorities={priorities}
          loading={false}
          error={error}
          emptyHint="Keine Treffer."
          // Die Suche kennt keinen Filter - sie sucht ueber alles.
          filtered={false}
          onResetFilter={() => {}}
          // Abgehaktes gleitet hier bewusst nicht hinaus: ein Treffer soll
          // stehen bleiben, man hat ihn ja gerade gesucht.
          leaving={new Set()}
          onToggle={(t: Task) => void after(setTaskStatus(t.id, t.status === "open"))}
          onDuplicate={(id) => void after(duplicateTask(id))}
          onDelete={(id) => void after(deleteTask(id))}
          onOpen={(t: Task) => void openDetail(t.id)}
        />
      )}
    </div>
  );
}
