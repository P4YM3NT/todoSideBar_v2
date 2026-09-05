import { useEffect, useState } from "react";
import { Search, Settings2 } from "lucide-react";
import { type Task, type View, viewKey } from "@/lib/tasks";
import { openDetail } from "@/lib/detail";
import { DEFAULT_VIEW } from "@/lib/lastView";
import { type Priority, onQuickAdd } from "@/lib/ipc";
import type { Theme } from "@/lib/theme";
import { ThemeToggle } from "@/components/ThemeToggle";
import { ICON_ACTION } from "@/lib/styles";
import { useTasks } from "@/lib/useTasks";
import { QuickAdd } from "@/components/QuickAdd";
import { SearchView } from "@/components/SearchView";
import { TaskList } from "@/components/TaskList";
import { ViewSwitcher } from "@/components/ViewSwitcher";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

const EMPTY_HINT: Record<string, string> = {
  inbox: "Nichts in der Inbox.",
  today: "Für heute ist nichts geplant.",
  upcoming: "Keine späteren Termine.",
  done: "Noch nichts erledigt.",
};

const TITLE: Record<string, string> = {
  inbox: "Inbox",
  today: "Heute",
  upcoming: "Demnächst",
  done: "Erledigt",
};

type Props = {
  view: View;
  priorities: Priority[];
  showSubtasks: boolean;
  theme: Theme;
  /** Für den Tooltip an "Aufgabe hinzufügen". `null`, solange nicht geladen. */
  addHotkey: string | null;
  onThemeChange: (theme: Theme) => void;
  onViewChange: (view: View) => void;
  onOpenSettings: () => void;
};

export function Sidebar({
  view,
  priorities,
  showSubtasks,
  theme,
  addHotkey,
  onThemeChange,
  onViewChange,
  onOpenSettings,
}: Props) {
  const {
    tasks,
    projects,
    counts,
    loading,
    error,
    leaving,
    reload,
    add,
    addProject,
    toggle,
    duplicate,
    remove,
  } = useTasks(view, showSubtasks);
  const [searching, setSearching] = useState(false);
  /** Hochgezaehlt vom Hotkey - `QuickAdd` klappt bei jeder Aenderung auf. */
  const [addSignal, setAddSignal] = useState(0);

  // ⌘F wie überall sonst auch. Das Detailfenster ist ein eigenes Fenster und
  // belegt die Kombination für sich - hier kommt sich nichts in die Quere.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        setSearching(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Der globale Hotkey zum Anlegen. Rust hat die Sidebar schon aufgeklappt -
  // hier muss nur noch der Weg zum Eingabefeld frei sein.
  useEffect(() => {
    const sub = onQuickAdd(() => {
      // In der Suche und in "Erledigt" gibt es kein Eingabefeld; ohne diesen
      // Wechsel liefe der Hotkey dort ins Leere.
      setSearching(false);
      if (view.kind === "done") onViewChange(DEFAULT_VIEW);
      setAddSignal((n) => n + 1);
    });
    return () => void sub.then((un) => un());
  }, [view, onViewChange]);

  // Ein gemerktes Projekt kann inzwischen geloescht sein - dann zurueck auf Heute.
  useEffect(() => {
    if (
      view.kind === "project" &&
      projects.length > 0 &&
      !projects.some((p) => p.id === view.value)
    ) {
      onViewChange(DEFAULT_VIEW);
    }
  }, [view, projects, onViewChange]);

  if (searching) {
    return (
      <SearchView
        priorities={priorities}
        onClose={() => setSearching(false)}
        onChanged={() => void reload()}
      />
    );
  }

  const key = viewKey(view);
  const title =
    TITLE[key] ??
    (view.kind === "project"
      ? (projects.find((p) => p.id === view.value)?.name ?? "Projekt")
      : "Projekt");

  return (
    <div className="flex h-full flex-col gap-4">
      <header className="flex shrink-0 items-center justify-between">
        <h1 className="line-clamp-1 text-sm font-medium">{title}</h1>
        <div className="flex items-center">
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => setSearching(true)}
                  aria-label="Suchen"
                  className={ICON_ACTION}
                />
              }
            >
              <Search className="size-4" />
            </TooltipTrigger>
            <TooltipContent>Suchen · ⌘F</TooltipContent>
          </Tooltip>
          <ThemeToggle theme={theme} onChange={onThemeChange} />
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={onOpenSettings}
            aria-label="Einstellungen"
            className={ICON_ACTION}
          >
            <Settings2 className="size-4" />
          </Button>
        </div>
      </header>

      <div className="shrink-0">
        <ViewSwitcher
          value={view}
          projects={projects}
          counts={counts}
          onChange={onViewChange}
          onCreateProject={addProject}
        />
      </div>

      <Separator className="shrink-0" />

      <TaskList
        tasks={tasks}
        priorities={priorities}
        loading={loading}
        error={error}
        emptyHint={EMPTY_HINT[key] ?? "Keine Aufgaben in diesem Projekt."}
        onToggle={(t) => void toggle(t)}
        leaving={leaving}
        onDuplicate={(id) => void duplicate(id)}
        onDelete={(id) => void remove(id)}
        onOpen={(t: Task) => void openDetail(t.id)}
      />

      {/* Erledigt ist eine Rueckschau - dort etwas anzulegen ergibt keinen Sinn. */}
      {view.kind !== "done" && (
        <div className="shrink-0 space-y-3">
          <Separator className="bg-border/80" />
          <QuickAdd
            view={view}
            projects={projects}
            hotkey={addHotkey}
            openSignal={addSignal}
            onCreate={add}
          />
        </div>
      )}
    </div>
  );
}
