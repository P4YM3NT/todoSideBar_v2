import { useEffect, useState } from "react";
import { ACCENT_PRESETS, isValidColor, readableOn } from "@/lib/accent";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { cn } from "@/lib/utils";

/** Feste Farben ohne den "Standard"-Eintrag - eine Stufe braucht immer eine Farbe. */
const PRESETS = ACCENT_PRESETS.filter(
  (p): p is { value: string; label: string } => p.value !== null,
);

type Props = {
  value: string;
  onChange: (color: string) => void;
};

/**
 * Kompakter Farbwaehler fuer Listen: ein Feld, das ein Popover mit Auswahl und
 * freier Eingabe oeffnet. Die grosse `ColorField`-Reihe waere pro Zeile zu viel.
 */
export function ColorSwatch({ value, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(value);
  const valid = isValidColor(draft);

  useEffect(() => {
    if (open) setDraft(value);
  }, [open, value]);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={
          <button
            type="button"
            aria-label="Farbe wählen"
            style={{ backgroundColor: value, color: readableOn(value) }}
            className="size-6 shrink-0 rounded-md border border-border outline-none
                       transition-colors focus-visible:ring-3 focus-visible:ring-ring/50"
          />
        }
      />

      <PopoverContent align="start" className="w-56 space-y-2 p-3">
        <div className="flex flex-wrap gap-2">
          {PRESETS.map((preset) => (
            <button
              key={preset.label}
              type="button"
              title={preset.label}
              aria-label={preset.label}
              onClick={() => {
                onChange(preset.value);
                setOpen(false);
              }}
              style={{ backgroundColor: preset.value }}
              className={cn(
                "size-6 rounded-md border outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                value === preset.value ? "border-ring" : "border-border",
              )}
            />
          ))}
        </div>

        <div className="flex items-center gap-2">
          <input
            type="color"
            aria-label="Farbe wählen"
            value={valid ? draft : "#000000"}
            onChange={(e) => {
              setDraft(e.target.value);
              onChange(e.target.value);
            }}
            className="size-7 shrink-0 cursor-pointer rounded-md border border-border bg-transparent p-0"
          />
          <Input
            value={draft}
            spellCheck={false}
            placeholder="#ef4444"
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && valid) {
                onChange(draft.trim());
                setOpen(false);
              }
            }}
            className="h-7 font-mono text-xs"
          />
        </div>

        <Button
          size="sm"
          className="w-full"
          disabled={!valid}
          onClick={() => {
            onChange(draft.trim());
            setOpen(false);
          }}
        >
          Übernehmen
        </Button>
      </PopoverContent>
    </Popover>
  );
}
