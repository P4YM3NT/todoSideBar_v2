import { useState } from "react";
import {
  Bell,
  CalendarDays,
  ChevronRight,
  FolderOpen,
  Flag,
  Keyboard,
  ListFilter,
  Palette,
  SlidersHorizontal,
} from "lucide-react";
import type { AppSettings } from "@/lib/ipc";
import { BackgroundPage } from "@/components/settings/BackgroundPage";
import { ControlsPage } from "@/components/settings/ControlsPage";
import { FiltersPage } from "@/components/settings/FiltersPage";
import { GeneralPage } from "@/components/settings/GeneralPage";
import { NotificationsPage } from "@/components/settings/NotificationsPage";
import { CalendarPage } from "@/components/settings/CalendarPage";
import { PrioritiesPage } from "@/components/settings/PrioritiesPage";
import { ProjectsPage } from "@/components/settings/ProjectsPage";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

/** Bewusst als Liste angelegt: die App soll weiter anpassbar werden, neue
 *  Bereiche kommen hier dazu, ohne die Navigation umzubauen. */
const PAGES = [
  {
    id: "general",
    label: "Allgemein",
    hint: "Akzentfarbe",
    icon: SlidersHorizontal,
  },
  {
    id: "background",
    label: "Hintergrund",
    hint: "Erscheinungsbild, Deckkraft",
    icon: Palette,
  },
  {
    id: "projects",
    label: "Projekte",
    hint: "Anlegen, Farben, Unteraufgaben",
    icon: FolderOpen,
  },
  {
    id: "priorities",
    label: "Prioritäten",
    hint: "Stufen, Farben, Bezeichnungen",
    icon: Flag,
  },
  {
    id: "filters",
    label: "Filter & Sortierung",
    hint: "Presets",
    icon: ListFilter,
  },
  {
    id: "notifications",
    label: "Benachrichtigungen",
    hint: "Native Popups, E-Mail, Erinnerungszeit",
    icon: Bell,
  },
  {
    id: "controls",
    label: "Bedienung",
    hint: "Hotkeys, Griff",
    icon: Keyboard,
  },
  {
    id: "calendar",
    label: "Kalendersync",
    hint: "Apple, Google, automatische Synchronisierung",
    icon: CalendarDays,
  },
] as const;

type PageId = (typeof PAGES)[number]["id"];

type Props = {
  /** Einzige Quelle ist `App` - eine eigene Kopie hier wuerde Aenderungen von
   *  aussen (etwa das Ziehen der Fensterkante) nicht mitbekommen. */
  settings: AppSettings;
  onClose: () => void;
  onChange: (settings: AppSettings) => void;
};

export function Settings({ settings, onClose, onChange }: Props) {
  const [page, setPage] = useState<PageId | null>(null);

  const apply = onChange;
  const back = () => setPage(null);

  if (page === "background") {
    return <BackgroundPage settings={settings} onChange={apply} onBack={back} />;
  }
  if (page === "general") {
    return <GeneralPage settings={settings} onChange={apply} onBack={back} />;
  }
  if (page === "projects") {
    return <ProjectsPage settings={settings} onChange={apply} onBack={back} />;
  }
  if (page === "priorities") {
    return <PrioritiesPage settings={settings} onChange={apply} onBack={back} />;
  }
  if (page === "filters") {
    return <FiltersPage settings={settings} onChange={apply} onBack={back} />;
  }
  if (page === "controls") {
    return <ControlsPage settings={settings} onChange={apply} onBack={back} />;
  }
  if (page === "notifications") {
    return <NotificationsPage settings={settings} onChange={apply} onBack={back} />;
  }
  if (page === "calendar") {
    return <CalendarPage settings={settings} onChange={apply} onBack={back} />;
  }

  return (
    <div className="flex h-full flex-col gap-4">
      <header className="flex items-center justify-between">
        <h1 className="text-sm font-medium">Einstellungen</h1>
        <Button variant="ghost" size="sm" onClick={onClose}>
          Fertig
        </Button>
      </header>

      <Separator />

      <nav className="space-y-1">
        {PAGES.map(({ id, label, hint, icon: Icon }) => (
          <button
            key={id}
            type="button"
            onClick={() => setPage(id)}
            className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left
                       transition-colors outline-none hover:bg-accent/50
                       focus-visible:ring-3 focus-visible:ring-ring/50"
          >
            <Icon className="size-4 shrink-0 text-muted-foreground" />
            <span className="min-w-0 flex-1">
              <span className="block text-sm">{label}</span>
              <span className="block text-xs text-muted-foreground">{hint}</span>
            </span>
            <ChevronRight className="size-4 shrink-0 text-muted-foreground" />
          </button>
        ))}
      </nav>
    </div>
  );
}
