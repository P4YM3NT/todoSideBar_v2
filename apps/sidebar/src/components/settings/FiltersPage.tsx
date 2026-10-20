import { Plus, Trash2 } from "lucide-react";
import {
  DEFAULT_SORT,
  DUE_LABEL,
  EMPTY_FILTER,
  type FilterPreset,
  SORT_LABEL,
  STATUS_LABEL,
  activeFilterCount,
} from "@/lib/filters";
import { type AppSettings, setFilterPresets } from "@/lib/ipc";
import { SubPage } from "@/components/Shared";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

/**
 * Beschreibt in einer Zeile, was ein Preset tut. Die Feinheiten stellt man im
 * Filter-Panel ein, wo man die Wirkung sofort an der Liste sieht - hier geht es
 * nur um Benennen, Anlegen und Löschen.
 */
function describe(preset: FilterPreset): string {
  const parts: string[] = [SORT_LABEL[preset.sort.field]];

  const count = activeFilterCount(preset.filter);
  if (count === 0) {
    parts.push("kein Filter");
  } else {
    if (preset.filter.projects.length > 0) {
      parts.push(`${preset.filter.projects.length} Projekte`);
    }
    if (preset.filter.priorities.length > 0) {
      parts.push(`${preset.filter.priorities.length} Stufen`);
    }
    if (preset.filter.status !== "inherit") {
      parts.push(STATUS_LABEL[preset.filter.status]);
    }
    if (preset.filter.due !== "any") {
      parts.push(DUE_LABEL[preset.filter.due]);
    }
  }

  return parts.join(" · ");
}

export function FiltersPage({ settings, onChange, onBack }: Props) {
  const list = settings.filterPresets;

  const save = (next: FilterPreset[]) => {
    void setFilterPresets(next);
    onChange({ ...settings, filterPresets: next });
  };

  const add = () => {
    // Nie eine ID wiederverwenden: die gemerkte Auswahl je Ansicht zeigt
    // darauf, und eine recycelte ID böge sie stillschweigend um.
    const id = list.reduce((max, p) => Math.max(max, p.id), 0) + 1;
    save([
      ...list,
      {
        id,
        name: `Preset ${list.length + 1}`,
        filter: EMPTY_FILTER,
        sort: DEFAULT_SORT,
      },
    ]);
  };

  return (
    <SubPage title="Filter & Sortierung" onBack={onBack}>
      <p className="text-xs leading-snug text-muted-foreground">
        Presets sind gespeicherte Paare aus Filter und Sortierung. Wählen und
        einstellen lassen sie sich über das Symbol im Kopf der Liste; hier
        werden sie benannt, angelegt und gelöscht.
      </p>

      <div className="space-y-2">
        {list.map((preset) => (
          <div
            key={preset.id}
            className="space-y-1 rounded-md border border-border bg-background/40 px-2 py-2"
          >
            <div className="flex items-center gap-2">
              <Input
                value={preset.name}
                onChange={(e) =>
                  save(
                    list.map((p) =>
                      p.id === preset.id ? { ...p, name: e.target.value } : p,
                    ),
                  )
                }
                className="h-7 min-w-0 flex-1 text-sm"
              />
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label={`${preset.name} löschen`}
                onClick={() => save(list.filter((p) => p.id !== preset.id))}
                className="shrink-0 text-muted-foreground hover:text-destructive"
              >
                <Trash2 className="size-4" />
              </Button>
            </div>
            <p className="truncate px-2 text-xs text-muted-foreground">
              {describe(preset)}
            </p>
          </div>
        ))}
      </div>

      {list.length === 0 && (
        <p className="text-xs text-muted-foreground">
          Noch keine Presets. Eines entsteht auch, wenn du im Filter-Panel etwas
          einstellst und es dort sicherst.
        </p>
      )}

      <Button variant="outline" size="sm" onClick={add} className="w-full">
        <Plus className="size-4" />
        Preset anlegen
      </Button>
    </SubPage>
  );
}
