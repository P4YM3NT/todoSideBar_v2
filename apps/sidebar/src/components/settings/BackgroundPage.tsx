import {
  type AppSettings,
  setBackgroundAlpha,
  setBackgroundColor,
  setTheme,
} from "@/lib/ipc";
import { BACKGROUND_PRESETS, BACKGROUND_PRESETS_MORE } from "@/lib/accent";
import { ColorField } from "@/components/settings/ColorField";
import { type Theme, applyTheme } from "@/lib/theme";
import { Row, SubPage } from "@/components/settings/Shared";
import { Slider } from "@/components/ui/slider";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const THEME_LABELS: Record<string, string> = {
  light: "Hell",
  dark: "Dunkel",
  system: "Wie das System",
};

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

export function BackgroundPage({ settings, onChange, onBack }: Props) {
  return (
    <SubPage title="Hintergrund" onBack={onBack}>
      <Row label="Erscheinungsbild">
        <Select
          value={settings.theme}
          onValueChange={(v) => {
            const theme = (v ?? "light") as Theme;
            // Sofort anwenden, damit man die Wahl direkt sieht.
            applyTheme(theme);
            void setTheme(theme);
            onChange({ ...settings, theme });
          }}
        >
          <SelectTrigger size="sm" className="w-full">
            <SelectValue>
              {(value: string | null) => THEME_LABELS[value ?? "light"] ?? "Hell"}
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="light">Hell</SelectItem>
            <SelectItem value="dark">Dunkel</SelectItem>
            <SelectItem value="system">Wie das System</SelectItem>
          </SelectContent>
        </Select>
      </Row>

      <Row
        label="Farbe der Fläche"
        hint="Überschreibt die Farbe des Themes."
      >
        <ColorField
          presets={BACKGROUND_PRESETS}
          morePresets={BACKGROUND_PRESETS_MORE}
          value={settings.backgroundColor}
          fallbackClassName="bg-card"
          onChange={(color) => {
            void setBackgroundColor(color);
            onChange({ ...settings, backgroundColor: color });
          }}
        />
      </Row>

      <Row
        label={`Deckkraft: ${Math.round(settings.backgroundAlpha)} %`}
        hint="Wie stark die Fläche der Sidebar deckt. Niedriger heißt durchsichtiger."
      >
        <Slider
          value={[settings.backgroundAlpha]}
          min={20}
          max={100}
          step={1}
          onValueChange={(v) =>
            onChange({
              ...settings,
              backgroundAlpha: Array.isArray(v) ? v[0] : v,
            })
          }
          // Erst beim Loslassen speichern - waehrend des Ziehens waere das ein
          // Schreibvorgang pro Pixel.
          onValueCommitted={(v) =>
            void setBackgroundAlpha(Array.isArray(v) ? v[0] : v)
          }
        />
      </Row>
    </SubPage>
  );
}
