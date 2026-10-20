import { useState, type KeyboardEvent, type PointerEvent } from "react";
import { CalendarDays, Clock, Copy, GripVertical, Maximize2, MoreHorizontal, RotateCcw, Trash2 } from "lucide-react";
import type { Priority } from "@/lib/ipc";
import type { Task } from "@/lib/tasks";
import { scheduleLabel } from "@/lib/schedule";
import { ICON_ACTION } from "@/lib/styles";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import { DeleteTaskDialog } from "@/components/DeleteTaskDialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

/** Ganze Tage bis zum Datum; negativ heisst ueberfaellig. */
function daysUntil(iso: string): number {
  const date = new Date(`${iso}T00:00:00`);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  return Math.round((date.getTime() - today.getTime()) / 86_400_000);
}


/** Termine werden kurz und relativ gezeigt - in einer schmalen Spalte zaehlt jedes Zeichen. */
function formatDate(iso: string): string {
  const date = new Date(`${iso}T00:00:00`);
  const days = daysUntil(iso);

  if (days === 0) return "Heute";
  if (days === 1) return "Morgen";
  if (days === -1) return "Gestern";
  if (days < 0) return `${Math.abs(days)} T. überfällig`;
  if (days < 7) return date.toLocaleDateString("de-DE", { weekday: "short" });
  return date.toLocaleDateString("de-DE", { day: "2-digit", month: "2-digit" });
}

type Props = {
  task: Task;
  /** Frei definierbare Stufen aus den Einstellungen. */
  priorities: Priority[];
  /** Zeile gleitet gerade hinaus, weil sie abgehakt wurde. */
  leaving?: boolean;
  onToggle: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
  onOpen: () => void;
  onDragStart?: (event: PointerEvent<HTMLButtonElement>) => void;
  onReorderKeyDown?: (event: KeyboardEvent<HTMLButtonElement>) => void;
  onResetPosition?: () => void;
  dragOffset?: number;
  reorderDisabled?: boolean;
};

export function TaskRow({
  task,
  priorities,
  leaving = false,
  onToggle,
  onDuplicate,
  onDelete,
  onOpen,
  onDragStart,
  onReorderKeyDown,
  onResetPosition,
  dragOffset,
  reorderDisabled = false,
}: Props) {
  const [confirming, setConfirming] = useState(false);
  const done = task.status === "done";
  const date = task.dueOn ?? task.scheduledOn;
  const time = scheduleLabel(task);
  const overdue =
    !done && task.dueOn !== null && new Date(`${task.dueOn}T00:00:00`) < new Date();
  // Uhr signalisiert Dringlichkeit, Kalender nur einen Termin in der Zukunft.
  const urgent = !done && date !== null && daysUntil(date) <= 0;
  const DateIcon = urgent ? Clock : CalendarDays;
  // Kann fehlen, wenn die Stufe inzwischen geloescht wurde.
  const priority = priorities.find((p) => p.id === task.priority);
  // Ohne Beschreibung und Datum ist die Karte einzeilig. Dann muss der Inhalt
  // mittig sitzen: die Aktionssymbole sind 28 px hoch, der Titel nur 20 - mit
  // `items-start` klebte er oben und wirkte versetzt.
  const multiline = Boolean(task.description) || Boolean(date) || Boolean(time);

  return (
    <div
      title={priority && `Priorität ${priority.label}`}
      className={cn(
        "group flex cursor-pointer gap-3 rounded-md border border-border",
        multiline ? "items-start" : "items-center",
        "bg-background/40 px-3 py-2",
        // Prioritaet faerbt die linke Kante: als eigenes Symbol stand sie in
        // derselben schmalen Zeile wie das Datum und hat sie zugedraengt.
        priority && "border-l-2",
        // `translate`, nicht `transform`: Tailwind v4 setzt fuer translate-x-*
        // die eigenstaendige CSS-Eigenschaft `translate`. Steht die nicht in
        // der Transition-Liste, springt die Zeile ohne Animation weg.
        // Hoehe bleibt bewusst unanimiert - das wuerde jeden Frame das Layout
        // neu berechnen.
        "transition-[translate,opacity,background-color] duration-[380ms]",
        leaving
          ? "translate-x-full opacity-0 ease-[cubic-bezier(0.4,0,0.6,1)]"
          : "ease-[cubic-bezier(0.32,0.72,0,1)] hover:bg-accent/50",
        dragOffset !== undefined && "relative z-20 cursor-grabbing bg-accent shadow-md",
      )}
      // Nutzerfarbe, deshalb Inline-Style statt Klasse - wie bei Projektfarben.
      style={{
        borderLeftColor: priority?.color,
        transform: dragOffset !== undefined ? `translateY(${dragOffset}px)` : undefined,
      }}
    >
      {/* Auf die erste Textzeile ausgerichtet, nicht auf die ganze Karte. */}
      <Checkbox
        checked={done}
        onCheckedChange={onToggle}
        aria-label={done ? "Als offen markieren" : "Als erledigt markieren"}
        className={cn(
          multiline && "mt-0.5",
          "shrink-0 data-checked:border-(--checkbox-accent) data-checked:bg-(--checkbox-accent) data-checked:text-(--checkbox-accent-fg)",
        )}
      />

      <button
        type="button"
        onClick={onOpen}
        className="min-w-0 flex-1 space-y-1 rounded-md text-left outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
      >
        {/* Einzeilig mit Auslassungspunkten - eine zweite Zeile laesst die
            Zeilenhoehe springen und macht die Liste unruhig. */}
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

        {date && (
          <span
            className={cn(
              "flex items-center gap-1 text-xs",
              overdue ? "text-destructive" : "text-muted-foreground",
            )}
          >
            <DateIcon className="size-3 shrink-0" />
            {formatDate(date)}
          </span>
        )}
        {time && <span className="flex items-center gap-1 text-xs text-muted-foreground">
          <Clock className="size-3 shrink-0" />{time}
        </span>}
      </button>

      {/* `has-aria-expanded` haelt die Schaltflaechen sichtbar, solange das Menue
          offen ist - sonst blenden sie aus, sobald die Maus zum Menue wandert,
          und das offene Menue haengt an nichts mehr. */}
      <div className={cn(
        "flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100 has-aria-expanded:opacity-100",
        dragOffset !== undefined && "opacity-100",
      )}>
        {onDragStart ? (
          <Button
            variant="ghost"
            size="icon-sm"
            disabled={reorderDisabled}
            onPointerDown={onDragStart}
            onKeyDown={onReorderKeyDown}
            aria-label={`${task.title} verschieben`}
            title={`${onResetPosition ? "Feste Position · " : ""}Ziehen oder mit ↑/↓ verschieben`}
            className={cn(ICON_ACTION, "touch-none cursor-grab active:cursor-grabbing", onResetPosition && "text-(--checkbox-accent)")}
          >
            <GripVertical className="size-4" />
          </Button>
        ) : (
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={onOpen}
            aria-label="Details öffnen"
            className={ICON_ACTION}
          >
            <Maximize2 className="size-4" />
          </Button>
        )}
        {/* Alles Weitere ins Menue: eine dritte Schaltflaeche haette in einer
            220 px schmalen Spalte den Titel auf wenige Zeichen zusammengedrueckt. */}
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Weitere Aktionen"
                className={ICON_ACTION}
              />
            }
          >
            <MoreHorizontal className="size-4" />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-auto">
            {onDragStart && (
              <DropdownMenuItem onClick={onOpen}>
                <Maximize2 />
                Details öffnen
              </DropdownMenuItem>
            )}
            {onResetPosition && (
              <DropdownMenuItem onClick={onResetPosition}>
                <RotateCcw />
                Position zurücksetzen
              </DropdownMenuItem>
            )}
            <DropdownMenuItem onClick={onDuplicate}>
              <Copy />
              Duplizieren
            </DropdownMenuItem>
            <DropdownMenuItem variant="destructive" onClick={() => setConfirming(true)}>
              <Trash2 />
              Löschen
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <DeleteTaskDialog
        task={confirming ? task : null}
        onOpenChange={setConfirming}
        onConfirm={onDelete}
      />
    </div>
  );
}
