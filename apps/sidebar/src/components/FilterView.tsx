import { useState } from "react";
import { ArrowDown, ArrowUp, ChevronRight } from "lucide-react";
import {
  DEFAULT_SORT,
  DUE_LABEL,
  type DueFilter,
  EMPTY_FILTER,
  type FilterPreset,
  SORT_FIELDS,
  SORT_LABEL,
  STATUS_LABEL,
  type SortField,
  type StatusFilter,
  type TaskFilter,
  type ViewSetting,
  isSettingActive,
  sameFilter,
  sameSort,
  toggleIn,
} from "@/lib/filters";
import type { Priority } from "@/lib/ipc";
import type { Project } from "@/lib/tasks";
import { Row, SubPage } from "@/components/Shared";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";

/** Stufe 0 heißt "ohne Priorität" - so steht es auch in der Datenbank. */
const NO_PRIORITY = 0;

type Props = {
  setting: ViewSetting;
  presets: FilterPreset[];
  projects: Project[];
  priorities: Priority[];
  onChange: (setting: ViewSetting) => void;
  onSavePreset: (name: string, setting: ViewSetting) => void;
  onClose: () => void;
};

/** Eine ankreuzbare Zeile - Kästchen und Beschriftung sind ein Klickziel. */
function CheckRow({
  label,
  checked,
  onToggle,
  swatch,
}: {
  label: string;
  checked: boolean;
  onToggle: () => void;
  swatch?: string | null;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-accent/50">
      <Checkbox checked={checked} onCheckedChange={onToggle} className="shrink-0" />
      {/* Projekt- und Prioritätsfarben sind Nutzerdaten und kommen als
          Inline-Style aus der Datenbank, nicht als Tailwind-Klasse. */}
      {swatch && (
        <span
          className="size-2 shrink-0 rounded-full"
          style={{ backgroundColor: swatch }}
        />
      )}
      <span className="min-w-0 flex-1 truncate text-sm">{label}</span>
    </label>
  );
}

/** Zeile, die eine Ebene tiefer führt - wie in den Einstellungen. */
function NavRow({
  label,
  value,
  onClick,
}: {
  label: string;
  value: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left
                 transition-colors outline-none hover:bg-accent/50
                 focus-visible:ring-3 focus-visible:ring-ring/50"
    >
      <span className="min-w-0 flex-1 truncate text-sm">{label}</span>
      <span className="shrink-0 text-xs text-muted-foreground">{value}</span>
      <ChevronRight className="size-4 shrink-0 text-muted-foreground" />
    </button>
  );
}

/** Kompakte Auswahl mit lesbarer Beschriftung statt des rohen Werts. */
function LabeledSelect<T extends string>({
  value,
  labels,
  order,
  onChange,
}: {
  value: T;
  labels: Record<T, string>;
  order: T[];
  onChange: (value: T) => void;
}) {
  return (
    <Select value={value} onValueChange={(v) => onChange((v ?? value) as T)}>
      <SelectTrigger size="sm" className="w-full text-sm">
        {/* Ohne Darstellungsfunktion zeigt Base UI den rohen Wert. */}
        <SelectValue>{(v: string | null) => labels[(v ?? value) as T]}</SelectValue>
      </SelectTrigger>
      <SelectContent alignItemWithTrigger={false} className="max-w-(--available-width)">
        {order.map((key) => (
          <SelectItem key={key} value={key}>
            {labels[key]}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

/**
 * Filter und Sortierung in einer Ansicht.
 *
 * Ersetzt den Inhalt der Sidebar, statt in einem Popover zu hängen - genauso
 * macht es `SearchView`. In einer 220 px breiten Spalte wäre ein Popover
 * ohnehin so breit wie die Spalte selbst.
 *
 * Die Auswahl von Projekten und Prioritäten ist eine zweite Ebene darin -
 * dasselbe Muster wie die Unterseiten der Einstellungen.
 */
export function FilterView({
  setting,
  presets,
  projects,
  priorities,
  onChange,
  onSavePreset,
  onClose,
}: Props) {
  const [section, setSection] = useState<"projects" | "priorities" | null>(null);
  const [presetName, setPresetName] = useState("");

  const { filter, sort } = setting;

  // Jede Änderung von Hand löst die Bindung ans Preset: sonst stünde oben ein
  // Name, der nicht mehr beschreibt, was eingestellt ist.
  const edit = (patch: Partial<ViewSetting>) =>
    onChange({ ...setting, presetId: null, ...patch });

  const setFilter = (patch: Partial<TaskFilter>) =>
    edit({ filter: { ...filter, ...patch } });

  if (section === "projects") {
    return (
      <SubPage title="Projekte" onBack={() => setSection(null)}>
        <p className="text-xs leading-snug text-muted-foreground">
          Nichts angekreuzt heißt: alle Projekte.
        </p>
        <div className="-mx-2">
          <CheckRow
            label="Ohne Projekt"
            checked={filter.projects.includes(null)}
            onToggle={() => setFilter({ projects: toggleIn(filter.projects, null) })}
          />
          {projects.map((p) => (
            <CheckRow
              key={p.id}
              label={p.name}
              swatch={p.color}
              checked={filter.projects.includes(p.id)}
              onToggle={() => setFilter({ projects: toggleIn(filter.projects, p.id) })}
            />
          ))}
        </div>
      </SubPage>
    );
  }

  if (section === "priorities") {
    return (
      <SubPage title="Prioritäten" onBack={() => setSection(null)}>
        <p className="text-xs leading-snug text-muted-foreground">
          Nichts angekreuzt heißt: alle Stufen.
        </p>
        <div className="-mx-2">
          <CheckRow
            label="Ohne Priorität"
            checked={filter.priorities.includes(NO_PRIORITY)}
            onToggle={() =>
              setFilter({ priorities: toggleIn(filter.priorities, NO_PRIORITY) })
            }
          />
          {priorities.map((p) => (
            <CheckRow
              key={p.id}
              label={p.label}
              swatch={p.color}
              checked={filter.priorities.includes(p.id)}
              onToggle={() =>
                setFilter({ priorities: toggleIn(filter.priorities, p.id) })
              }
            />
          ))}
        </div>
      </SubPage>
    );
  }

  const count = (n: number, total: number) =>
    n === 0 ? "alle" : `${n} von ${total}`;

  const active = isSettingActive(setting);
  const matching = presets.find((p) => p.id === setting.presetId);
  // "Angepasst" nur zeigen, wenn wirklich etwas eingestellt ist - beim leeren
  // Standard wäre der Hinweis sinnlos.
  const modified = !matching && active;

  return (
    <SubPage title="Filter & Sortierung" onBack={onClose}>
      {presets.length > 0 && (
        <Row label="Preset">
          <div className="-mx-2">
            {presets.map((preset) => {
              const chosen =
                setting.presetId === preset.id ||
                (sameFilter(filter, preset.filter) && sameSort(sort, preset.sort));
              return (
                <button
                  key={preset.id}
                  type="button"
                  onClick={() =>
                    onChange({
                      filter: preset.filter,
                      sort: preset.sort,
                      presetId: preset.id,
                    })
                  }
                  className={cn(
                    "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
                    chosen
                      ? "bg-accent font-medium text-accent-foreground"
                      : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
                  )}
                >
                  <span
                    className={cn(
                      "size-2 shrink-0 rounded-full border border-current",
                      chosen && "bg-current",
                    )}
                  />
                  <span className="min-w-0 flex-1 truncate text-left">
                    {preset.name}
                  </span>
                </button>
              );
            })}
          </div>

          {modified && (
            <div className="space-y-2 rounded-md border border-border bg-background/40 px-3 py-2">
              <p className="text-xs text-muted-foreground">
                Angepasst — als eigenes Preset sichern?
              </p>
              <div className="flex items-center gap-2">
                <Input
                  value={presetName}
                  onChange={(e) => setPresetName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && presetName.trim()) {
                      onSavePreset(presetName.trim(), setting);
                      setPresetName("");
                    }
                  }}
                  placeholder="Name"
                  className="h-7 flex-1 text-sm"
                />
                <Button
                  size="sm"
                  disabled={!presetName.trim()}
                  onClick={() => {
                    onSavePreset(presetName.trim(), setting);
                    setPresetName("");
                  }}
                >
                  Sichern
                </Button>
              </div>
            </div>
          )}
        </Row>
      )}

      <Separator />

      <Row
        label="Sortieren nach"
        hint={
          sort.field === "manual"
            ? "Die Reihenfolge, die die Ansicht selbst vorgibt."
            : undefined
        }
      >
        <div className="flex items-center gap-2">
          <div className="min-w-0 flex-1">
            <LabeledSelect<SortField>
              value={sort.field}
              labels={SORT_LABEL}
              order={SORT_FIELDS}
              onChange={(field) => edit({ sort: { ...sort, field } })}
            />
          </div>
          <Button
            variant="outline"
            size="icon-sm"
            // Bei "Reihenfolge" gibt die Ansicht die Richtung vor - ein
            // Umschalter, der nichts bewirkt, wäre irreführend.
            disabled={sort.field === "manual"}
            aria-label={
              sort.direction === "asc" ? "Absteigend sortieren" : "Aufsteigend sortieren"
            }
            onClick={() =>
              edit({
                sort: {
                  ...sort,
                  direction: sort.direction === "asc" ? "desc" : "asc",
                },
              })
            }
          >
            {sort.direction === "asc" ? (
              <ArrowUp className="size-4" />
            ) : (
              <ArrowDown className="size-4" />
            )}
          </Button>
        </div>
      </Row>

      <Separator />

      <Row label="Filter">
        <div className="-mx-2">
          <NavRow
            label="Projekte"
            value={count(filter.projects.length, projects.length + 1)}
            onClick={() => setSection("projects")}
          />
          <NavRow
            label="Prioritäten"
            value={count(filter.priorities.length, priorities.length + 1)}
            onClick={() => setSection("priorities")}
          />
        </div>

        <LabeledSelect<StatusFilter>
          value={filter.status}
          labels={STATUS_LABEL}
          order={["inherit", "open", "done", "all"]}
          onChange={(status) => setFilter({ status })}
        />

        <LabeledSelect<DueFilter>
          value={filter.due}
          labels={DUE_LABEL}
          order={["any", "overdue", "today", "week", "noDate"]}
          onChange={(due) => setFilter({ due })}
        />
      </Row>

      <Separator />

      <Button
        variant="outline"
        size="sm"
        className="w-full"
        disabled={!active}
        onClick={() =>
          onChange({ filter: EMPTY_FILTER, sort: DEFAULT_SORT, presetId: null })
        }
      >
        Zurücksetzen
      </Button>
    </SubPage>
  );
}
