import { useCallback, useEffect, useRef, useState } from "react";
import {
  CalendarClock,
  Flag,
  FolderOpen,
  ChevronLeft,
  Loader2,
  Maximize2,
  Minimize2,
  Timer,
} from "lucide-react";
import {
  type Project,
  type Task,
  type TaskFields,
  createTask,
  listProjects,
  listTasks,
  updateTask,
} from "@/lib/tasks";
import { type Priority, getSettings, onThemeChanged } from "@/lib/ipc";
import { applyTheme } from "@/lib/theme";
import { useDetailBackground } from "@/lib/useDetailBackground";
import { openDetail } from "@/lib/detail";
import {
  type DetailTarget,
  closeDetail,
  getDetailTarget,
  showDetail,
  toggleDetailMaximized,
  notifyTasksChanged,
  onDetailTarget,
} from "@/lib/detail";
import { AutoTextarea } from "@/components/AutoTextarea";
import { DateField } from "@/components/DateField";
import { SubtaskList } from "@/components/SubtaskList";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

const INBOX = "__inbox__";
const NO_PRIORITY = "0";

/**
 * Ohne Rahmen und Flaeche - der Text soll wirken, nicht das Eingabefeld.
 *
 * Der Fokusring bleibt (das Design-System verbietet, ihn zu entfernen), aber
 * duenn. Das negative Aussenmass gleicht die Innenpolsterung wieder aus, damit
 * der Text buendig steht und der Ring trotzdem Abstand zu ihm hat.
 */
const BARE =
  "rounded-md border-0 bg-transparent px-2 py-1 -mx-2 shadow-none " +
  "focus-visible:ring-1 dark:bg-transparent";

/**
 * Wartet, bis das eben Gerenderte auch gezeichnet ist.
 *
 * Zwei Frames: nach dem ersten steht das Layout, nach dem zweiten ist es auf
 * dem Schirm. Ein einzelner reicht nicht - das Fenster ginge dann gelegentlich
 * noch leer auf.
 *
 * Mit Zeitgrenze, weil das Fenster hier noch versteckt ist: WebKit drosselt
 * `requestAnimationFrame` in unsichtbaren Fenstern und stellt es unter
 * Umstaenden ganz ein. Ohne die Grenze haenge das Einblenden dann bis zur
 * Notbremse in Rust - jedes Oeffnen braeuchte anderthalb Sekunden.
 */
const PAINT_TIMEOUT_MS = 120;

const nextPaint = () =>
  Promise.race([
    new Promise<void>((resolve) =>
      requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
    ),
    new Promise<void>((resolve) => setTimeout(resolve, PAINT_TIMEOUT_MS)),
  ]);

const emptyFields = (): TaskFields => ({
  title: "",
  description: null,
  notes: null,
  projectId: null,
  scheduledOn: null,
  dueOn: null,
  priority: 0,
  estimateMinutes: null,
});

const fieldsOf = (task: Task): TaskFields => ({
  title: task.title,
  description: task.description,
  notes: task.notes,
  projectId: task.projectId,
  scheduledOn: task.scheduledOn,
  dueOn: task.dueOn,
  priority: task.priority,
  estimateMinutes: task.estimateMinutes,
});

/** Einheitlicher Chip fuer alle Metadaten - eine Zeile statt beschrifteter Kaesten. */
function Chip({
  icon: Icon,
  children,
  muted,
}: {
  icon: React.ComponentType<{ className?: string }>;
  children: React.ReactNode;
  muted?: boolean;
}) {
  return (
    <span
      className={cn("flex items-center gap-2", muted && "text-muted-foreground")}
    >
      <Icon className="size-3.5 shrink-0" />
      {children}
    </span>
  );
}

export function TaskDetail() {
  const [taskId, setTaskId] = useState<string | null>(null);
  const [fields, setFields] = useState<TaskFields>(emptyFields);
  const [projects, setProjects] = useState<Project[]>([]);
  const [priorities, setPriorities] = useState<Priority[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);
  const [full, setFull] = useState(false);
  /** Gesetzt, wenn die offene Aufgabe eine Unteraufgabe ist - sonst kaeme man
   *  nach dem Oeffnen nicht mehr zur Hauptaufgabe zurueck. */
  const [parent, setParent] = useState<Task | null>(null);
  const scroller = useRef<HTMLDivElement>(null);

  /** Zaehlt jeden Ladevorgang mit, damit ein ueberholter nichts mehr setzt. */
  const run = useRef(0);

  const load = useCallback(async (target: DetailTarget) => {
    const mine = ++run.current;

    setError(null);
    setTaskId(target.taskId);
    // Ohne das behaelt das Fenster die Scrollposition der vorigen Aufgabe -
    // beim Oeffnen einer Unteraufgabe startet man dann mitten im Dokument.
    scroller.current?.scrollTo({ top: 0 });

    setParent(null);

    try {
      // Alles in einem Rutsch statt nacheinander. Projekte und Stufen muessen
      // bei *jedem* Oeffnen mit: das Fenster wird beim Schliessen nur
      // versteckt, sonst stuenden hier die Werte von vorhin. Und sie muessen
      // vor dem Einblenden da sein - sonst erscheint das Fenster und Projekt
      // und Prioritaet tragen sich sichtbar nach.
      //
      // Es gibt bewusst kein `get_task`-Kommando: die Liste ist lokal und
      // klein, ein zweiter Befehl nur zum Nachladen einer Zeile lohnt nicht.
      // Unteraufgaben sind in `all` enthalten, deshalb das Flag.
      const [projectList, settings, all] = await Promise.all([
        listProjects(),
        getSettings(),
        target.taskId === null
          ? Promise.resolve(null)
          : listTasks({ kind: "all" }, true),
      ]);

      // Ein zweites Oeffnen hat dieses hier ueberholt - dessen Daten gelten.
      if (mine !== run.current) return;

      setProjects(projectList);
      setPriorities(settings.priorities);

      if (all === null) {
        setFields({
          ...emptyFields(),
          title: target.prefillTitle ?? "",
          projectId: target.prefillProjectId ?? null,
        });
      } else {
        const found = all.find((t) => t.id === target.taskId);
        if (!found) {
          setError("Aufgabe nicht gefunden.");
        } else {
          setFields(fieldsOf(found));
          setParent(all.find((t) => t.id === found.parentId) ?? null);
        }
      }
    } catch (e) {
      if (mine === run.current) setError(String(e));
    } finally {
      if (mine === run.current) {
        setReady(true);
        // Auch im Fehlerfall einblenden: ein Fenster, das gar nicht erscheint,
        // sieht aus, als haette der Klick nicht gewirkt.
        await nextPaint();
        await showDetail();
      }
    }
  }, []);

  useDetailBackground();

  useEffect(() => {
    // Beim Start selbst abholen - ein Event kann zu frueh kommen.
    void getDetailTarget().then((id) =>
      load({ taskId: id, prefillTitle: null, prefillProjectId: null }),
    );
    const sub = onDetailTarget((t) => void load(t));
    return () => void sub.then((un) => un());
  }, [load]);

  // Das Fenster wird beim Schliessen nur versteckt und ueberlebt darum jeden
  // Themenwechsel in der Sidebar. Ohne diesen Horcher behielte es das Theme
  // von dem Moment, in dem es zum ersten Mal geoeffnet wurde.
  useEffect(() => {
    const sub = onThemeChanged(applyTheme);
    return () => void sub.then((un) => un());
  }, []);

  const set = <K extends keyof TaskFields>(key: K, value: TaskFields[K]) =>
    setFields((f) => ({ ...f, [key]: value }));

  const save = useCallback(async () => {
    if (!fields.title.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const payload = { ...fields, title: fields.title.trim() };
      if (taskId === null) await createTask(payload);
      else await updateTask(taskId, payload);
      await notifyTasksChanged();
      await closeDetail();
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  }, [fields, busy, taskId]);

  // Tastatur wie in nativen Dialogen: sichern mit Cmd+Enter, schliessen mit Esc.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
        e.preventDefault();
        void save();
      }
      if (e.key === "Escape") void closeDetail();
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        void toggleDetailMaximized().then(setFull);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [save]);

  if (!ready) {
    return (
      <div className="flex h-full items-center justify-center bg-background">
        <Loader2 className="size-4 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const project = projects.find((p) => p.id === fields.projectId);
  const priority = priorities.find((p) => p.id === fields.priority);

  return (
    <div className="relative flex h-full flex-col bg-background text-foreground">
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={full ? "Fenster verkleinern" : "Fenster vergrößern"}
              onClick={() => void toggleDetailMaximized().then(setFull)}
              className="absolute right-4 top-4 z-10 text-muted-foreground"
            />
          }
        >
          {full ? <Minimize2 className="size-4" /> : <Maximize2 className="size-4" />}
        </TooltipTrigger>
        <TooltipContent>{full ? "Verkleinern" : "Vergrößern"} · ⌘F</TooltipContent>
      </Tooltip>

      <div ref={scroller} className="flex-1 overflow-y-auto px-8 py-6">
        <div className="mx-auto flex h-full max-w-xl flex-col gap-6">
          {parent && (
            <button
              type="button"
              onClick={() => void openDetail(parent.id)}
              className="-ml-2 flex w-fit items-center gap-1 rounded-md px-2 py-1 text-xs
                         text-muted-foreground outline-none transition-colors
                         hover:text-foreground focus-visible:ring-1 focus-visible:ring-ring/50"
            >
              <ChevronLeft className="size-3 shrink-0" />
              <span className="truncate">{parent.title}</span>
            </button>
          )}

          <div className="space-y-1">
            <Input
              value={fields.title}
              onChange={(e) => set("title", e.target.value)}
              placeholder="Titel"
              className={cn(BARE, "h-auto text-lg font-semibold")}
            />
            <AutoTextarea
              value={fields.description ?? ""}
              onChange={(e) => set("description", e.target.value || null)}
              placeholder="Kurze Beschreibung"
              className={cn(BARE, "min-h-0 text-sm text-muted-foreground")}
            />
          </div>

          {/* Weich gehalten: eine kraeftige Linie wuerde das Dokument wieder in
              Formularbloecke zerschneiden. */}
          <Separator className="bg-border/60" />

          {/* Metadaten als Chips in einer Zeile - beschriftete Kaesten machen
              aus einer Aufgabe ein Formular. */}
          <div className="flex flex-wrap items-center gap-2">
            <Select
              value={fields.projectId ?? INBOX}
              onValueChange={(v) => set("projectId", !v || v === INBOX ? null : v)}
            >
              <SelectTrigger size="sm" className="w-auto">
                <Chip icon={FolderOpen} muted={!project}>
                  {project?.name ?? "Inbox"}
                </Chip>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={INBOX}>Inbox</SelectItem>
                {projects.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select
              value={String(fields.priority)}
              onValueChange={(v) => set("priority", Number(v ?? NO_PRIORITY))}
            >
              <SelectTrigger size="sm" className="w-auto">
                <Chip icon={Flag} muted={!priority}>
                  <span className="flex items-center gap-2">
                    {priority && (
                      <span
                        aria-hidden
                        className="size-2 shrink-0 rounded-md"
                        style={{ backgroundColor: priority.color }}
                      />
                    )}
                    {priority?.label ?? "Priorität"}
                  </span>
                </Chip>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={NO_PRIORITY}>Keine</SelectItem>
                {priorities.map((p) => (
                  <SelectItem key={p.id} value={String(p.id)}>
                    <span className="flex items-center gap-2">
                      <span
                        aria-hidden
                        className="size-2 shrink-0 rounded-md"
                        style={{ backgroundColor: p.color }}
                      />
                      {p.label}
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <DateField
              value={fields.scheduledOn}
              placeholder="Geplant"
              onChange={(v) => set("scheduledOn", v)}
            />
            <DateField
              value={fields.dueOn}
              placeholder="Fällig"
              onChange={(v) => set("dueOn", v)}
              icon={CalendarClock}
            />

            <span className="flex items-center gap-1 rounded-md border border-border px-2 text-sm">
              <Timer className="size-3.5 shrink-0 text-muted-foreground" />
              <Input
                type="number"
                min={0}
                step={5}
                value={fields.estimateMinutes ?? ""}
                onChange={(e) =>
                  set("estimateMinutes", e.target.value ? Number(e.target.value) : null)
                }
                placeholder="–"
                aria-label="Aufwand in Minuten"
                className={cn(BARE, "mx-0 h-7 w-10 px-0 text-sm")}
              />
              <span className="text-xs text-muted-foreground">min</span>
            </span>
          </div>

          {/* Erst nach dem Speichern moeglich: eine Unteraufgabe braucht eine
              Eltern-ID, die es bei einer noch nicht angelegten Aufgabe nicht gibt. */}
          {taskId && (
            <>
              <Separator className="bg-border/60" />
              <SubtaskList parentId={taskId} />
            </>
          )}

          <Separator className="bg-border/60" />

          <Textarea
            value={fields.notes ?? ""}
            onChange={(e) => set("notes", e.target.value || null)}
            placeholder="Notizen…"
            className={cn(BARE, "min-h-40 flex-1 resize-none text-sm leading-relaxed")}
          />
        </div>
      </div>

      <footer className="flex items-center justify-between gap-4 border-t border-border px-8 py-4">
        <span className="text-xs text-destructive">{error}</span>
        <div className="flex items-center gap-2">
          {/* Kuerzel im Tooltip statt dauerhaft daneben - permanent angezeigt
              ist es Beiwerk, das man nach dem ersten Mal nicht mehr liest. */}
          <Tooltip>
            <TooltipTrigger
              render={
                <Button variant="ghost" size="sm" onClick={() => void closeDetail()} />
              }
            >
              Abbrechen
            </TooltipTrigger>
            <TooltipContent>Esc</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  size="sm"
                  onClick={() => void save()}
                  disabled={!fields.title.trim() || busy}
                />
              }
            >
              {taskId === null ? "Anlegen" : "Speichern"}
            </TooltipTrigger>
            <TooltipContent>⌘ ↵</TooltipContent>
          </Tooltip>
        </div>
      </footer>
    </div>
  );
}
