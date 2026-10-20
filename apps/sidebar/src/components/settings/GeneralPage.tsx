import { type AppSettings, setCheckboxColor } from "@/lib/ipc";
import { ACCENT_PRESETS } from "@/lib/accent";
import { ColorField } from "@/components/settings/ColorField";
import { Row, SubPage } from "@/components/Shared";

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

export function GeneralPage({ settings, onChange, onBack }: Props) {
  return (
    <SubPage title="Allgemein" onBack={onBack}>
      <Row
        label="Akzentfarbe"
        hint="Gilt für abgehakte Aufgaben, Symbole und Regler."
      >
        <ColorField
          presets={ACCENT_PRESETS}
          value={settings.checkboxColor}
          fallbackClassName="bg-primary text-primary-foreground"
          onChange={(color) => {
            void setCheckboxColor(color);
            onChange({ ...settings, checkboxColor: color });
          }}
        />
      </Row>
    </SubPage>
  );
}
