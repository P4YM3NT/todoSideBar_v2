import { useEffect, useState } from "react";
import { RotateCcw } from "lucide-react";
import {
  type AppSettings,
  resetSidebarWidth,
  setAddHotkey,
  setHandleVisible,
  setHotkey,
  commitSidebarWidth,
} from "@/lib/ipc";
import { HotkeyRecorder } from "@/components/HotkeyRecorder";
import { Row, SubPage } from "@/components/settings/Shared";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

export function ControlsPage({ settings, onChange, onBack }: Props) {
  // Eigener Entwurf, damit man das Feld leeren und neu tippen kann, ohne dass
  // bei jedem Tastendruck das Fenster springt.
  const [draft, setDraft] = useState(String(Math.round(settings.width)));

  useEffect(() => {
    setDraft(String(Math.round(settings.width)));
  }, [settings.width]);

  const commitWidth = () => {
    const value = Number(draft);
    if (!Number.isFinite(value) || value <= 0) {
      setDraft(String(Math.round(settings.width)));
      return;
    }
    void commitSidebarWidth(value).then((width) =>
      onChange({ ...settings, width }),
    );
  };

  return (
    <SubPage title="Bedienung" onBack={onBack}>
      <Row
        label="Breite der Sidebar"
        hint="In Punkten. Lässt sich auch durch Ziehen an der linken Kante ändern."
      >
        <div className="flex items-center gap-2">
          <Input
            type="number"
            min={220}
            step={10}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commitWidth}
            onKeyDown={(e) => {
              if (e.key === "Enter") commitWidth();
            }}
            className="h-7 flex-1"
          />
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="Auf Standard zurücksetzen"
            title="Auf 15 % der Bildschirmbreite zurücksetzen"
            onClick={async () => {
              const width = await resetSidebarWidth();
              onChange({ ...settings, width });
            }}
          >
            <RotateCcw className="size-4" />
          </Button>
        </div>
      </Row>

      <Separator />

      <Row
        label="Globaler Hotkey"
        hint="Öffnet und schließt die Sidebar aus jeder App heraus."
      >
        <HotkeyRecorder
          value={settings.hotkey}
          onChange={async (accelerator) => {
            await setHotkey(accelerator);
            onChange({ ...settings, hotkey: accelerator });
          }}
        />
      </Row>

      <Separator />

      <Row
        label="Hotkey zum Anlegen"
        hint="Klappt die Sidebar auf und setzt den Cursor gleich ins Eingabefeld."
      >
        <HotkeyRecorder
          value={settings.addHotkey}
          onChange={async (accelerator) => {
            await setAddHotkey(accelerator);
            onChange({ ...settings, addHotkey: accelerator });
          }}
        />
      </Row>

      <Separator />

      <Row
        label="Griff am Bildschirmrand"
        hint={
          settings.handleVisible
            ? "Der schmale Streifen rechts klappt die Sidebar auf."
            : "Vollständig ausgeblendet. Die Sidebar erreichst du weiterhin über den Hotkey und das Symbol in der Menüleiste."
        }
      >
        <Switch
          checked={settings.handleVisible}
          onCheckedChange={(v) => {
            void setHandleVisible(v);
            onChange({ ...settings, handleVisible: v });
          }}
        />
      </Row>
    </SubPage>
  );
}
