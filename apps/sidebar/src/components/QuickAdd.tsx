import { useEffect, useRef, useState } from "react";
import { Maximize2, Plus } from "lucide-react";
import type { NewTask, Project, View } from "@/lib/tasks";
import { defaultProjectFor } from "@/lib/tasks";
import { openDetail } from "@/lib/detail";
import { ICON_ACTION } from "@/lib/styles";
import { pretty } from "@/lib/hotkey";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const INBOX = "__inbox__";

type Props = {
  view: View;
  projects: Project[];
  /** Globaler Hotkey zum Anlegen - nur für den Tooltip. `null` = noch unbekannt. */
  hotkey: string | null;
  /** Jede Änderung klappt die Zeile auf. Kommt vom Hotkey in `Sidebar`. */
  openSignal: number;
  onCreate: (task: NewTask) => Promise<void>;
};

/**
 * Schnellanlegen: "+" klappt eine Zeile auf, in die direkt getippt wird.
 * Kein Formular, kein Dialog - der Weg vom Gedanken zur Aufgabe soll kurz sein.
 * Für alles Größere gibt es das Detailfenster.
 */
export function QuickAdd({ view, projects, hotkey, openSignal, onCreate }: Props) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [projectId, setProjectId] = useState<string>(INBOX);
  const [busy, setBusy] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const boxRef = useRef<HTMLDivElement>(null);

  // Vorbelegung folgt der Ansicht: in Projekt X angelegt heißt Projekt X -
  // änderbar bleibt es trotzdem.
  useEffect(() => {
    if (open) {
      setProjectId(defaultProjectFor(view) ?? INBOX);
      inputRef.current?.focus();
    }
  }, [open, view]);

  // Ein Zaehler statt eines Schalters: zweimal denselben Hotkey zu druecken
  // soll die Zeile auch dann wieder oeffnen, wenn sie zwischendurch zuging.
  // Der Startwert 0 bedeutet "noch nie gedrueckt" und darf nichts ausloesen.
  useEffect(() => {
    if (openSignal === 0) return;
    setOpen(true);
    inputRef.current?.focus();
  }, [openSignal]);

  const close = () => {
    setOpen(false);
    setTitle("");
    setMenuOpen(false);
  };

  // Ein Klick daneben klappt die Zeile wieder zu - aber nur, solange nichts
  // getippt wurde. Getipptes stehen zu lassen ist wichtiger als das aufgeraeumte
  // Bild; verworfen wird bewusst per Escape.
  //
  // `pointerdown` statt `blur`: die Zeile soll auch dann zugehen, wenn der Klick
  // ins Leere geht und gar kein neues Element den Fokus bekommt. Das Select-Popup
  // haengt im Portal und damit ausserhalb von `boxRef` - solange es offen ist,
  // zaehlt deshalb kein Klick als "daneben".
  useEffect(() => {
    if (!open) return;

    const onPointerDown = (e: PointerEvent) => {
      if (menuOpen || title.trim()) return;
      const target = e.target as Node | null;
      if (target && boxRef.current?.contains(target)) return;
      close();
    };

    document.addEventListener("pointerdown", onPointerDown, true);
    return () => document.removeEventListener("pointerdown", onPointerDown, true);
  }, [open, menuOpen, title]);

  // Getipptes mitnehmen, damit nichts doppelt eingegeben werden muss.
  const expand = () => {
    void openDetail(null, {
      title: title.trim() || null,
      projectId: projectId === INBOX ? null : projectId,
    });
    close();
  };

  const submit = async () => {
    if (!title.trim() || busy) return;
    setBusy(true);
    try {
      await onCreate({
        title: title.trim(),
        projectId: projectId === INBOX ? null : projectId,
      });
      // Offen lassen und leeren: mehrere Aufgaben hintereinander sind der Normalfall.
      setTitle("");
      inputRef.current?.focus();
    } finally {
      setBusy(false);
    }
  };

  if (!open) {
    return (
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setOpen(true)}
              // Gleiche Hover-Farbe wie Navigationszeile und Aufgabenzeile: der
              // Ausloeser ist eine Zeile unter der Liste, kein eigenstaendiger Knopf.
              className="w-full justify-start px-2 text-muted-foreground hover:bg-accent/50 hover:text-foreground dark:hover:bg-accent/50"
            />
          }
        >
          <Plus />
          Aufgabe hinzufügen
        </TooltipTrigger>
        {/* Solange die Einstellungen nicht geladen sind, steht der Hotkey noch
            nicht fest - dann lieber ohne als mit leerem Zusatz. */}
        <TooltipContent>
          {hotkey ? `Aufgabe hinzufügen · ${pretty(hotkey)}` : "Aufgabe hinzufügen"}
        </TooltipContent>
      </Tooltip>
    );
  }

  return (
    // Rahmen, Rundung und Innenabstand sind die der Aufgabenzeile - was hier
    // entsteht, sieht aus wie das, was gleich in der Liste steht.
    //
    // Innen ist alles rahmenlos: Feld und Projektwahl brachten je eine eigene
    // Kante mit, drei ineinanderliegende Rahmen auf 220 px wirken verbaut.
    // Der Fokusring des Feldes liegt deshalb auf der Box - sichtbar bleibt er,
    // nur eine Ebene weiter aussen.
    //
    // Schmaler als die ueblichen 3 px: das Feld bekommt den Fokus schon beim
    // Aufklappen, der Ring steht also dauerhaft und nicht nur im Moment des
    // Tabbens. Wie beim Feld fuer Unteraufgaben reicht hier 1 px.
    <div
      ref={boxRef}
      className="animate-in fade-in slide-in-from-bottom-1 space-y-2 rounded-md border border-border bg-background/40 px-3 py-2 transition-colors duration-150 has-[input:focus-visible]:border-ring has-[input:focus-visible]:ring-1 has-[input:focus-visible]:ring-ring/50"
    >
      <Input
        ref={inputRef}
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onKeyDown={(e) => {
          // Enter legt an, ⌘↵ macht daraus die ausfuehrliche Fassung - dieselbe
          // Kombination, die im Detailfenster speichert.
          if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
            e.preventDefault();
            expand();
            return;
          }
          if (e.key === "Enter") {
            e.preventDefault();
            void submit();
          }
          if (e.key === "Escape") close();
        }}
        placeholder="Was ist zu tun?"
        className="h-7 rounded-md border-0 bg-transparent px-0 text-sm shadow-none focus-visible:ring-0 dark:bg-transparent"
      />

      <div className="flex items-center gap-1">
        <Select
          value={projectId}
          onValueChange={(v) => setProjectId(v ?? INBOX)}
          open={menuOpen}
          onOpenChange={setMenuOpen}
        >
          {/* Negativer Rand haelt den Text buendig unter dem Eingabefeld -
              der Innenabstand des Ausloesers wuerde ihn sonst einruecken. */}
          <SelectTrigger
            size="sm"
            className="-ml-2 min-w-0 rounded-md border-0 bg-transparent px-2 text-xs text-muted-foreground hover:bg-accent/50 hover:text-foreground dark:bg-transparent dark:hover:bg-accent/50"
          >
            {/* Ohne Darstellungsfunktion zeigt Base UI den rohen Wert - also
                `__inbox__` statt "Inbox". */}
            <SelectValue>
              {(value: string | null) =>
                value && value !== INBOX
                  ? (projects.find((p) => p.id === value)?.name ?? "Inbox")
                  : "Inbox"
              }
            </SelectValue>
          </SelectTrigger>
          {/* Base UI legt die Liste sonst *ueber* den Ausloeser, damit der
              gewaehlte Eintrag genau auf ihm liegt. Bei einem so schmalen,
              linksbuendigen Ausloeser draengt sie damit an den Fensterrand.
              Als gewoehnliches Aufklappmenue darunter bleibt sie im Panel;
              der kleine Versatz stellt die Eintraege unter den Ausloesertext. */}
          <SelectContent
            align="start"
            alignOffset={2}
            alignItemWithTrigger={false}
            className="w-auto min-w-36 max-w-(--available-width)"
          >
            <SelectGroup>
              <SelectItem value={INBOX}>Inbox</SelectItem>
              {projects.map((p) => (
                <SelectItem key={p.id} value={p.id}>
                  {p.name}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>

        <div className="ml-auto flex shrink-0 items-center gap-1">
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label="Ausführlich anlegen"
                  className={ICON_ACTION}
                  onClick={expand}
                />
              }
            >
              <Maximize2 />
            </TooltipTrigger>
            <TooltipContent>Ausführlich anlegen · ⌘ ↵</TooltipContent>
          </Tooltip>

          <Button size="sm" onClick={() => void submit()} disabled={!title.trim() || busy}>
            Anlegen
          </Button>
        </div>
      </div>
    </div>
  );
}
