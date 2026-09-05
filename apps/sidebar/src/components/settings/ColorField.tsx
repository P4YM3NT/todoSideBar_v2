import { useEffect, useState } from "react";
import { Check, ChevronDown, Pipette } from "lucide-react";
import { isValidColor, readableOn, toHex } from "@/lib/accent";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { cn } from "@/lib/utils";

type Preset = { value: string | null; label: string };

type Props = {
  presets: Preset[];
  /** Weitere Farben hinter dem Aufklappen. Leer lassen, wenn es keine gibt. */
  morePresets?: { value: string; label: string }[];
  value: string | null;
  /** Wie ein Feld ohne eigene Farbe aussieht (Themefarbe). */
  fallbackClassName: string;
  onChange: (value: string | null) => void;
};

/**
 * Feste Felder plus freie Eingabe. Geprueft wird mit `CSS.supports` - damit
 * gilt genau das, was der Browser auch wirklich darstellen kann: Hex, rgb(),
 * rgba(), hsl(), Farbnamen.
 */
export function ColorField({
  presets,
  morePresets = [],
  value,
  fallbackClassName,
  onChange,
}: Props) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState("");
  // Aufgeklappt lassen, solange eine der weiteren Farben aktiv ist - sonst
  // verschwindet die aktuelle Auswahl aus dem Blick.
  const [expanded, setExpanded] = useState(() =>
    morePresets.some((p) => p.value === value),
  );

  const isCustom =
    value !== null &&
    !presets.some((p) => p.value === value) &&
    !morePresets.some((p) => p.value === value);
  const valid = isValidColor(draft);
  // Der native Waehler akzeptiert ausschliesslich #rrggbb. Alles andere - auch
  // eine gueltige rgba()-Eingabe - wird auf den aufgeloesten Wert abgebildet.
  const hexDraft = valid ? toHex(draft.trim()) : "#000000";

  // Beim Oeffnen mit dem aktuellen Wert starten, damit man ihn sieht und
  // anpassen kann, statt ihn neu tippen zu muessen.
  useEffect(() => {
    if (open) setDraft(value ?? "");
  }, [open, value]);

  const swatch = (
    key: string,
    color: string | null,
    label: string,
    active: boolean,
    onClick: () => void,
    children?: React.ReactNode,
  ) => (
    <button
      key={key}
      type="button"
      title={label}
      aria-label={label}
      aria-pressed={active}
      onClick={onClick}
      style={color ? { backgroundColor: color, color: readableOn(color) } : undefined}
      className={cn(
        "flex size-6 items-center justify-center rounded-md border transition-colors outline-none",
        "focus-visible:ring-3 focus-visible:ring-ring/50",
        active ? "border-ring" : "border-border",
        !color && fallbackClassName,
      )}
    >
      {children ?? (active && <Check className="size-3" />)}
    </button>
  );

  const row = (
    <div className="flex flex-wrap items-center gap-2">
      {presets.map((preset) =>
        swatch(
          preset.label,
          preset.value,
          preset.label,
          value === preset.value,
          () => onChange(preset.value),
        ),
      )}

      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger
          render={
            <button
              type="button"
              title="Eigene Farbe"
              aria-label="Eigene Farbe"
              aria-pressed={isCustom}
              style={
                isCustom && value
                  ? { backgroundColor: value, color: readableOn(value) }
                  : undefined
              }
              className={cn(
                "flex size-6 items-center justify-center rounded-md border transition-colors outline-none",
                "focus-visible:ring-3 focus-visible:ring-ring/50",
                isCustom ? "border-ring" : "border-border border-dashed",
              )}
            />
          }
        >
          <Pipette className="size-3" />
        </PopoverTrigger>

        <PopoverContent align="start" className="w-56 space-y-2 p-3">
          {/* Nativer Farbwaehler: oeffnet die Farbauswahl des Betriebssystems,
              inklusive Pipette. Eine eigene Flaeche mit Sattigungsfeld und
              Farbtonregler waere mehr Code bei weniger Funktion - und eine
              fertige Komponente dafuer gibt es in der Bibliothek nicht.
              Er kann nur Hex, deshalb bleibt das Textfeld fuer rgba(). */}
          <div className="flex items-center gap-2">
            <input
              type="color"
              aria-label="Farbe wählen"
              value={hexDraft}
              onChange={(e) => setDraft(e.target.value)}
              className="size-7 shrink-0 cursor-pointer rounded-md border border-border bg-transparent p-0"
            />
            <span className="text-xs leading-snug text-muted-foreground">
              Hex, rgb() oder rgba()
            </span>
          </div>
          <Input
            value={draft}
            autoFocus
            spellCheck={false}
            placeholder="#1c1c1e"
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && valid) {
                onChange(draft.trim());
                setOpen(false);
              }
              if (e.key === "Escape") setOpen(false);
            }}
            className="h-7 font-mono text-xs"
          />
          <div className="flex items-center gap-2">
            <span
              aria-hidden
              className="size-6 shrink-0 rounded-md border border-border"
              style={valid ? { backgroundColor: draft.trim() } : undefined}
            />
            <Button
              size="sm"
              className="flex-1"
              disabled={!valid}
              onClick={() => {
                onChange(draft.trim());
                setOpen(false);
              }}
            >
              Übernehmen
            </Button>
          </div>
          {draft.trim() !== "" && !valid && (
            <span className="text-xs text-destructive">Keine gültige Farbe.</span>
          )}
        </PopoverContent>
      </Popover>
    </div>
  );

  if (morePresets.length === 0) return row;

  return (
    <Collapsible open={expanded} onOpenChange={setExpanded} className="space-y-2">
      {row}

      <CollapsibleTrigger
        render={
          <button
            type="button"
            className="flex items-center gap-1 text-xs text-muted-foreground
                       transition-colors outline-none hover:text-foreground
                       focus-visible:ring-3 focus-visible:ring-ring/50 rounded-md"
          />
        }
      >
        <ChevronDown
          className={cn("size-3 transition-transform", expanded && "rotate-180")}
        />
        {expanded ? "Weniger Farben" : "Mehr Farben"}
      </CollapsibleTrigger>

      <CollapsibleContent>
        <div className="flex flex-wrap items-center gap-2 pt-1">
          {morePresets.map((preset) =>
            swatch(
              preset.label,
              preset.value,
              preset.label,
              value === preset.value,
              () => onChange(preset.value),
            ),
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}
