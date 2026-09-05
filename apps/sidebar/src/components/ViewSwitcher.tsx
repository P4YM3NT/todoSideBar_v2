import { CalendarClock, CheckCheck, ChevronDown, Inbox, Plus, Sun } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { Project, View } from "@/lib/tasks";
import { viewKey } from "@/lib/tasks";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { projectIcon } from "@/lib/projectIcons";
import { cn } from "@/lib/utils";

const FIXED = [
  { view: { kind: "inbox" } as View, label: "Inbox", icon: Inbox },
  { view: { kind: "today" } as View, label: "Heute", icon: Sun },
  { view: { kind: "upcoming" } as View, label: "Demnächst", icon: CalendarClock },
  { view: { kind: "done" } as View, label: "Erledigt", icon: CheckCheck },
];

type Props = {
  value: View;
  projects: Project[];
  /** Offene Aufgaben je Projekt-ID. */
  counts: Record<string, number>;
  onChange: (view: View) => void;
  onCreateProject: (name: string) => Promise<void>;
};

export function ViewSwitcher({
  value,
  projects,
  counts,
  onChange,
  onCreateProject,
}: Props) {
  const active = viewKey(value);
  // Aufgeklappt lassen, solange ein Projekt aktiv ist - sonst verschwindet die
  // gerade offene Ansicht aus dem Blick.
  const [open, setOpen] = useState(value.kind === "project");
  /** Der getippte Projektname; `null` heisst: das Feld ist gar nicht da. */
  const [draft, setDraft] = useState<string | null>(null);
  const adding = draft !== null;
  const draftRef = useRef<HTMLInputElement>(null);

  // Anlegen klappt die Liste immer auf - sonst entstuende das Projekt
  // ungesehen hinter einer zugeklappten Zeile.
  const startAdding = () => {
    setOpen(true);
    setDraft("");
  };

  useEffect(() => {
    if (adding) draftRef.current?.focus();
  }, [adding]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        startAdding();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const submitDraft = async () => {
    const name = draft?.trim();
    if (!name) return;
    // Leeren und offen lassen: mehrere Projekte hintereinander sind der Normalfall.
    setDraft("");
    await onCreateProject(name);
  };

  const row = (
    key: string,
    label: string,
    onClick: () => void,
    leading: React.ReactNode,
    trailing?: React.ReactNode,
  ) => (
    <button
      key={key}
      type="button"
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
        active === key
          ? "bg-accent font-medium text-accent-foreground"
          : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
      )}
    >
      {leading}
      <span className="min-w-0 flex-1 truncate text-left">{label}</span>
      {trailing}
    </button>
  );

  return (
    <nav className="space-y-1">
      {FIXED.map(({ view, label, icon: Icon }) => {
        const key = viewKey(view);
        return row(
          key,
          label,
          () => onChange(view),
          // Symbol erbt die Textfarbe der Zeile - neutral, nicht in der Akzentfarbe.
          <Icon className="size-4 shrink-0" />,
        );
      })}

      {/* Auch ohne Projekte sichtbar: sonst gaebe es keinen Weg, das erste
          anzulegen, ausser ueber die Einstellungen. */}
      <Separator className="my-2 bg-border/60" />

      {/* Nur die Projekte klappen zu - die festen Ansichten bleiben immer
          erreichbar, sie sind der haeufigste Einstieg. */}
      <Collapsible open={open} onOpenChange={setOpen}>
        <div className="flex items-center gap-1">
          <CollapsibleTrigger
            render={
              <button
                type="button"
                className="flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1.5
                           text-xs text-muted-foreground outline-none transition-colors
                           hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50"
              />
            }
          >
            <ChevronDown
              className={cn(
                "size-3 shrink-0 transition-transform",
                !open && "-rotate-90",
              )}
            />
            Projekte
          </CollapsibleTrigger>

          {/* Bewusst `icon-xs` statt der `icon-sm` aus der Kopfzeile: das
              Symbol gehoert zur kleinen Beschriftungszeile und wuerde sie
              in der Groesse der uebrigen Aktionen erschlagen. */}
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={startAdding}
                  aria-label="Projekt anlegen"
                  className="text-muted-foreground hover:bg-accent/50 hover:text-foreground dark:hover:bg-accent/50"
                />
              }
            >
              <Plus />
            </TooltipTrigger>
            <TooltipContent>Projekt anlegen · ⌘ ⇧ N</TooltipContent>
          </Tooltip>
        </div>

        <CollapsibleContent className="space-y-1 pt-1">
          {adding && (
            <div className="flex items-center gap-2 px-2">
              <Plus className="size-4 shrink-0 text-muted-foreground" />
              <Input
                ref={draftRef}
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    void submitDraft();
                  }
                  if (e.key === "Escape") setDraft(null);
                }}
                // Anders als beim Schnellanlegen reicht hier `onBlur`: es ist
                // kein Menue im Spiel, das den Fokus zwischendurch uebernimmt.
                onBlur={() => {
                  if (!draft.trim()) setDraft(null);
                }}
                placeholder="Projektname"
                className="h-6 rounded-md border-0 bg-transparent px-0 text-sm shadow-none focus-visible:ring-1 dark:bg-transparent"
              />
            </div>
          )}

          {projects.map((p) => {
            const Icon = projectIcon(p.icon);
            const count = counts[p.id] ?? 0;
            return row(
              `project:${p.id}`,
              p.name,
              () => onChange({ kind: "project", value: p.id }),
              // Projektfarben sind Nutzerdaten - Inline-Style, keine Klasse.
              <Icon
                className="size-4 shrink-0"
                style={{ color: p.color ?? undefined }}
              />,
              count > 0 && (
                <span className="shrink-0 text-xs text-muted-foreground">
                  {count}
                </span>
              ),
            );
          })}
        </CollapsibleContent>
      </Collapsible>
    </nav>
  );
}
