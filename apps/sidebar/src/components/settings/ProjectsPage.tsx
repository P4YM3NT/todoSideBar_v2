import { useEffect, useState } from "react";
import { Archive, ArchiveRestore, Plus } from "lucide-react";
import {
  type Project,
  NEW_PROJECT_COLOR,
  createProject,
  listProjects,
  projectOpenCounts,
  setProjectArchived,
  updateProject,
} from "@/lib/tasks";
import { type AppSettings, setShowSubtasks } from "@/lib/ipc";
import { notifyTasksChanged } from "@/lib/detail";
import { PROJECT_ICONS, projectIcon } from "@/lib/projectIcons";
import { ColorSwatch } from "@/components/settings/ColorSwatch";
import { Row, SubPage } from "@/components/settings/Shared";
import { DateField } from "@/components/DateField";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { cn } from "@/lib/utils";

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

export function ProjectsPage({ settings, onChange, onBack }: Props) {
  const [active, setActive] = useState<Project[]>([]);
  const [archived, setArchived] = useState<Project[]>([]);
  const [counts, setCounts] = useState<Record<string, number>>({});
  const [open, setOpen] = useState<string | null>(null);

  const reload = () => {
    void listProjects(false).then(setActive);
    void listProjects(true).then(setArchived);
    void projectOpenCounts().then(setCounts);
  };
  useEffect(reload, []);

  const save = async (p: Project) => {
    await updateProject(p.id, {
      name: p.name,
      description: p.description,
      color: p.color,
      icon: p.icon,
      dueOn: p.dueOn,
      showSubtasks: p.showSubtasks,
    });
    await notifyTasksChanged();
    reload();
  };

  const patch = (id: string, changes: Partial<Project>) =>
    setActive((prev) => prev.map((p) => (p.id === id ? { ...p, ...changes } : p)));

  const archive = async (id: string, value: boolean) => {
    await setProjectArchived(id, value);
    await notifyTasksChanged();
    reload();
  };

  return (
    <SubPage title="Projekte" onBack={onBack}>
      <Row
        label="Unteraufgaben in Listen"
        hint="Gilt für Inbox, Heute und Demnächst. Projekte können es einzeln überschreiben."
      >
        <Switch
          checked={settings.showSubtasks}
          onCheckedChange={(v) => {
            void setShowSubtasks(v);
            onChange({ ...settings, showSubtasks: v });
          }}
        />
      </Row>

      <Separator />

      <div className="space-y-2">
        {active.map((project) => {
          const Icon = projectIcon(project.icon);
          const expanded = open === project.id;
          return (
            <div
              key={project.id}
              className="rounded-md border border-border bg-background/40"
            >
              <div className="flex items-center gap-2 p-2">
                <ColorSwatch
                  value={project.color ?? NEW_PROJECT_COLOR}
                  onChange={(color) => {
                    patch(project.id, { color });
                    void save({ ...project, color });
                  }}
                />
                <Input
                  value={project.name}
                  onChange={(e) => patch(project.id, { name: e.target.value })}
                  onBlur={() => void save(project)}
                  placeholder="Projektname"
                  className="h-7 flex-1 text-sm"
                />
                <span className="w-6 shrink-0 text-right text-xs text-muted-foreground">
                  {counts[project.id] ?? 0}
                </span>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={expanded ? "Weniger" : "Mehr"}
                  onClick={() => setOpen(expanded ? null : project.id)}
                  className="text-muted-foreground"
                >
                  <Icon className="size-4" />
                </Button>
              </div>

              {expanded && (
                <div className="space-y-3 border-t border-border p-2">
                  <Input
                    value={project.description ?? ""}
                    onChange={(e) =>
                      patch(project.id, { description: e.target.value || null })
                    }
                    onBlur={() => void save(project)}
                    placeholder="Wofür ist das Projekt da?"
                    className="h-7 text-sm"
                  />

                  <div className="flex flex-wrap gap-1">
                    {Object.entries(PROJECT_ICONS).map(([name, Candidate]) => (
                      <Button
                        key={name}
                        variant="ghost"
                        size="icon-sm"
                        aria-label={name}
                        onClick={() => {
                          patch(project.id, { icon: name });
                          void save({ ...project, icon: name });
                        }}
                        className={cn(
                          "text-muted-foreground",
                          (project.icon ?? "folder") === name &&
                            "bg-accent text-foreground",
                        )}
                      >
                        <Candidate className="size-4" />
                      </Button>
                    ))}
                  </div>

                  <DateField
                    value={project.dueOn}
                    placeholder="Keine Frist"
                    onChange={(dueOn) => {
                      patch(project.id, { dueOn });
                      void save({ ...project, dueOn });
                    }}
                  />

                  <label className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
                    Unteraufgaben anzeigen
                    <Switch
                      checked={project.showSubtasks}
                      onCheckedChange={(v) => {
                        patch(project.id, { showSubtasks: v });
                        void save({ ...project, showSubtasks: v });
                      }}
                    />
                  </label>

                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => void archive(project.id, true)}
                    className="w-full justify-start text-muted-foreground"
                  >
                    <Archive />
                    Archivieren
                  </Button>
                </div>
              )}
            </div>
          );
        })}

        {active.length === 0 && (
          <p className="text-xs text-muted-foreground">
            Noch keine Projekte. Aufgaben ohne Projekt landen in der Inbox.
          </p>
        )}
      </div>

      <Button
        variant="outline"
        size="sm"
        className="w-full"
        onClick={async () => {
          await createProject(`Projekt ${active.length + 1}`, NEW_PROJECT_COLOR);
          await notifyTasksChanged();
          reload();
        }}
      >
        <Plus />
        Projekt anlegen
      </Button>

      {archived.length > 0 && (
        <Collapsible className="space-y-2">
          <CollapsibleTrigger
            render={
              <button
                type="button"
                className="text-xs text-muted-foreground outline-none transition-colors
                           hover:text-foreground focus-visible:ring-1 focus-visible:ring-ring/50"
              />
            }
          >
            Archiv ({archived.length})
          </CollapsibleTrigger>
          <CollapsibleContent className="space-y-2">
            {archived.map((project) => (
              <div
                key={project.id}
                className="flex items-center gap-2 rounded-md border border-border px-2 py-2"
              >
                <span className="min-w-0 flex-1 truncate text-sm text-muted-foreground">
                  {project.name}
                </span>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={`${project.name} zurückholen`}
                  onClick={() => void archive(project.id, false)}
                  className="text-muted-foreground"
                >
                  <ArchiveRestore className="size-4" />
                </Button>
              </div>
            ))}
          </CollapsibleContent>
        </Collapsible>
      )}
    </SubPage>
  );
}
