/** Auswahl fuer die Akzentfarbe. `null` = Themefarbe. */
export const ACCENT_PRESETS: { value: string | null; label: string }[] = [
  { value: null, label: "Standard" },
  { value: "#3b82f6", label: "Blau" },
  { value: "#22c55e", label: "Grün" },
  { value: "#f59e0b", label: "Orange" },
  { value: "#ef4444", label: "Rot" },
  { value: "#a855f7", label: "Violett" },
  { value: "#14b8a6", label: "Türkis" },
  { value: "#ec4899", label: "Pink" },
];

/** Auswahl fuer die Hintergrundflaeche. `null` = Themefarbe. */
export const BACKGROUND_PRESETS: { value: string | null; label: string }[] = [
  { value: null, label: "Standard" },
  { value: "#ffffff", label: "Weiß" },
  { value: "#f5f5f4", label: "Papier" },
  { value: "#1c1c1e", label: "Anthrazit" },
  { value: "#0b1020", label: "Nachtblau" },
  { value: "#12211a", label: "Tannengrün" },
  { value: "#1e1526", label: "Aubergine" },
  { value: "#241a12", label: "Espresso" },
];

/**
 * Erweiterte Auswahl fuer die Flaeche - hinter dem Aufklappen.
 * Nach Helligkeit gruppiert: erst helle, dann mittlere, dann dunkle Toene.
 */
export const BACKGROUND_PRESETS_MORE: { value: string; label: string }[] = [
  { value: "#fefce8", label: "Vanille" },
  { value: "#f1f5f9", label: "Nebel" },
  { value: "#f0fdf4", label: "Minze" },
  { value: "#fdf2f8", label: "Rosé" },
  { value: "#eff6ff", label: "Himmel" },
  { value: "#faf5ff", label: "Flieder" },

  { value: "#57534e", label: "Stein" },
  { value: "#475569", label: "Schiefer" },
  { value: "#3f4a3c", label: "Moos" },
  { value: "#4c3d2e", label: "Leder" },
  { value: "#3b3b58", label: "Indigo" },
  { value: "#4a3550", label: "Pflaume" },

  { value: "#000000", label: "Schwarz" },
  { value: "#111827", label: "Mitternacht" },
  { value: "#0f172a", label: "Tiefsee" },
  { value: "#1a1a0f", label: "Olive" },
  { value: "#2a1215", label: "Bordeaux" },
  { value: "#0d1f1c", label: "Petrol" },
];

/** Loest eine beliebige CSS-Farbe in `r,g,b` auf - auch `rgba()` oder Namen. */
function toRgb(color: string): [number, number, number] | null {
  const el = document.createElement("span");
  el.style.color = color;
  document.body.appendChild(el);
  const computed = getComputedStyle(el).color;
  el.remove();

  const match = computed.match(/-?[\d.]+/g);
  if (!match || match.length < 3) return null;
  return [Number(match[0]), Number(match[1]), Number(match[2])];
}

/**
 * Schwarz oder Weiss - je nachdem, was auf der gewaehlten Farbe lesbar ist.
 * Ohne das verschwindet der weisse Haken auf hellen Farben.
 *
 * Relative Leuchtdichte nach WCAG.
 */
export function readableOn(color: string): string {
  const rgb = toRgb(color);
  if (!rgb) return "#ffffff";

  const channel = (v: number) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  const luminance =
    0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2]);
  return luminance > 0.45 ? "#000000" : "#ffffff";
}

/** Loest eine CSS-Farbe (auch `var(--token)`) in `[r,g,b]` auf. */
export function resolveRgb(color: string): [number, number, number] {
  return toRgb(color) ?? [255, 255, 255];
}

/** Ist die Farbe dunkel genug, dass heller Text darauf lesbar ist? */
export function isDarkColor(color: string): boolean {
  return readableOn(color) === "#ffffff";
}

/** Beliebige CSS-Farbe nach `#rrggbb` - der native Farbwaehler kennt nur das. */
export function toHex(color: string): string {
  const rgb = toRgb(color);
  if (!rgb) return "#000000";
  return `#${rgb.map((v) => Math.round(v).toString(16).padStart(2, "0")).join("")}`;
}

/** Prueft mit den echten Regeln des Browsers, ob die Eingabe eine Farbe ist. */
export const isValidColor = (value: string) =>
  value.trim() !== "" && CSS.supports("color", value.trim());
