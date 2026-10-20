import { GripVertical, Plus, Trash2 } from "lucide-react";
import { type AppSettings, type Priority, setPriorities } from "@/lib/ipc";
import { ColorSwatch } from "@/components/settings/ColorSwatch";
import { SubPage } from "@/components/Shared";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

const NEW_COLOR = "#6366f1";

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

export function PrioritiesPage({ settings, onChange, onBack }: Props) {
  const list = settings.priorities;

  const save = (next: Priority[]) => {
    void setPriorities(next);
    onChange({ ...settings, priorities: next });
  };

  const update = (id: number, patch: Partial<Priority>) =>
    save(list.map((p) => (p.id === id ? { ...p, ...patch } : p)));

  const add = () => {
    // Nie eine ID wiederverwenden: Aufgaben zeigen darauf, und eine recycelte
    // ID wuerde sie stillschweigend der neuen Stufe zuschlagen.
    const id = list.reduce((max, p) => Math.max(max, p.id), 0) + 1;
    save([...list, { id, label: `Stufe ${list.length + 1}`, color: NEW_COLOR }]);
  };

  return (
    <SubPage title="Prioritäten" onBack={onBack}>
      <p className="text-xs leading-snug text-muted-foreground">
        Die Reihenfolge bestimmt den Rang, von niedrig nach hoch. Beim Löschen
        einer Stufe verlieren die betroffenen Aufgaben ihre Priorität.
      </p>

      <div className="space-y-2">
        {list.map((priority, index) => (
          <div
            key={priority.id}
            className="flex items-center gap-2 rounded-md border border-border bg-background/40 px-2 py-2"
          >
            {/* Zeigt den Rang an; Umsortieren per Ziehen kommt spaeter. */}
            <span
              className="flex size-5 shrink-0 items-center justify-center text-xs text-muted-foreground"
              title={`Rang ${index + 1}`}
            >
              <GripVertical className="size-3" />
            </span>

            <ColorSwatch
              value={priority.color}
              onChange={(color) => update(priority.id, { color })}
            />

            <Input
              value={priority.label}
              onChange={(e) => update(priority.id, { label: e.target.value })}
              placeholder="Bezeichnung"
              className="h-7 flex-1 text-sm"
            />

            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`${priority.label} löschen`}
              onClick={() => save(list.filter((p) => p.id !== priority.id))}
              className="text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
            >
              <Trash2 className="size-4" />
            </Button>
          </div>
        ))}

        {list.length === 0 && (
          <p className="text-xs text-muted-foreground">
            Keine Stufen. Aufgaben haben dann keine Priorität.
          </p>
        )}
      </div>

      <Button variant="outline" size="sm" onClick={add} className="w-full">
        <Plus />
        Stufe hinzufügen
      </Button>
    </SubPage>
  );
}
