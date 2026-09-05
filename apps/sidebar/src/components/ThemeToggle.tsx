import type { Theme } from "@/lib/theme";
import { resolveTheme } from "@/lib/theme";
import { Button } from "@/components/ui/button";
import { ICON_ACTION } from "@/lib/styles";
import { cn } from "@/lib/utils";

/** Acht Strahlen, gleichmaessig verteilt. */
const RAYS = Array.from({ length: 8 }, (_, i) => i * 45);

type Props = {
  theme: Theme;
  onChange: (theme: Theme) => void;
};

/**
 * Sonne und Mond als **eine** Form, die sich umwandelt.
 *
 * Bewusste Ausnahme von der Lucide-Regel des Design-Systems: ein Formwechsel
 * laesst sich mit zwei fertigen Icons nicht bauen, nur ueberblenden - und genau
 * das sollte es nicht sein.
 *
 * Der Mond entsteht, indem eine zweite Scheibe per Maske in die Sonne faehrt und
 * die Sichel ausschneidet; gleichzeitig fahren die Strahlen ein. Animiert werden
 * ausschliesslich `transform` und `opacity`.
 */
export function ThemeToggle({ theme, onChange }: Props) {
  const dark = resolveTheme(theme) === "dark";

  return (
    <Button
      variant="ghost"
      size="icon-sm"
      onClick={() => onChange(dark ? "light" : "dark")}
      aria-label={dark ? "Helles Erscheinungsbild" : "Dunkles Erscheinungsbild"}
      className={cn("relative", ICON_ACTION)}
    >
      <svg
        viewBox="0 0 24 24"
        className="size-4 overflow-visible"
        aria-hidden="true"
      >
        <mask id="theme-toggle-crescent">
          <rect x="0" y="0" width="24" height="24" fill="white" />
          {/* Faehrt herein und schneidet die Sichel aus.
              Geometrie: Scheibe r=5.5 um (12,12), Maske r=5.6 um (15,9).
              Mittenabstand 4.24 - die Kreise schneiden sich (0.1 < 4.24 < 11.1)
              und es entstehen echte Hoerner. Sichelbreite = d + r1 - r2 = 4.14
              bei Durchmesser 11, also 38 %.
              Vorher lag die Maske bei (20,5) mit Abstand 10.6 und hat nur eine
              flache Delle in den Rand geknabbert. */}
          <circle
            cx="15"
            cy="9"
            r="5.6"
            fill="black"
            className="transition-transform duration-500 ease-[cubic-bezier(0.32,0.72,0,1)]"
            style={{ transform: dark ? "translate(0,0)" : "translate(8px,-8px)" }}
          />
        </mask>

        {/* Skaliert wird die *Gruppe*, nicht der Radius: so waechst das fertig
            maskierte Ergebnis mit und die berechnete Sichelform bleibt erhalten.
            Ein groesserer Radius allein wuerde die Maske nicht mitziehen. */}
        <g
          className="origin-center transition-transform duration-500 ease-[cubic-bezier(0.32,0.72,0,1)]"
          style={{ transform: dark ? "scale(1.3)" : "scale(1)" }}
        >
          <circle
            cx="12"
            cy="12"
            r="5.5"
            fill="currentColor"
            mask="url(#theme-toggle-crescent)"
          />
        </g>

        <g
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          className="origin-center transition-[transform,opacity] duration-500 ease-[cubic-bezier(0.32,0.72,0,1)]"
          style={{
            opacity: dark ? 0 : 1,
            transform: dark ? "rotate(60deg) scale(0.4)" : "rotate(0deg) scale(1)",
          }}
        >
          {RAYS.map((angle) => (
            <line
              key={angle}
              x1="12"
              y1="1.5"
              x2="12"
              y2="3.5"
              style={{ transform: `rotate(${angle}deg)`, transformOrigin: "center" }}
            />
          ))}
        </g>
      </svg>
    </Button>
  );
}
