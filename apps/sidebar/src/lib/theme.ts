export type Theme = "light" | "dark" | "system";

const MIRROR_KEY = "todosidebar.theme";
const QUERY = "(prefers-color-scheme: dark)";

let stopWatching: (() => void) | null = null;

/** Was gerade tatsaechlich angezeigt wird - "system" aufgeloest. */
export function resolveTheme(theme: Theme): "light" | "dark" {
  if (theme !== "system") return theme;
  return window.matchMedia(QUERY).matches ? "dark" : "light";
}

const TRANSITION_MS = 320;
let transitionTimer: ReturnType<typeof setTimeout> | null = null;

function setClass(dark: boolean) {
  const root = document.documentElement;
  if (root.classList.contains("dark") === dark) return;

  // Nur fuer die Dauer des Wechsels: sonst laege auf jedem Hover und jeder
  // Zustandsaenderung dieselbe Verzoegerung.
  root.classList.add("theme-transition");
  if (transitionTimer) clearTimeout(transitionTimer);
  transitionTimer = setTimeout(
    () => root.classList.remove("theme-transition"),
    TRANSITION_MS,
  );

  // shadcn definiert das dunkle Theme unter `.dark` und aktiviert es nicht
  // von selbst - ohne diese Klasse bleibt die App im hellen Theme.
  root.classList.toggle("dark", dark);
}

/**
 * Wendet das Thema an. Bei "system" wird auf Aenderungen der Systemeinstellung
 * gehoert, sonst nicht.
 */
export function applyTheme(theme: Theme) {
  stopWatching?.();
  stopWatching = null;

  if (theme === "system") {
    const query = window.matchMedia(QUERY);
    const onChange = (e: MediaQueryListEvent) => setClass(e.matches);
    setClass(query.matches);
    query.addEventListener("change", onChange);
    stopWatching = () => query.removeEventListener("change", onChange);
  } else {
    setClass(theme === "dark");
  }

  try {
    localStorage.setItem(MIRROR_KEY, theme);
  } catch {
    // Ohne Spiegel blitzt beim naechsten Start kurz das helle Theme auf -
    // mehr passiert nicht.
  }
}

/**
 * Sofort beim Start anwenden, noch bevor die Einstellungen aus der Datenbank
 * da sind. Ohne diesen Spiegel sieht man bei jedem Start kurz das helle Theme.
 */
export function applyMirroredTheme() {
  let theme: Theme = "light";
  try {
    const raw = localStorage.getItem(MIRROR_KEY);
    if (raw === "light" || raw === "dark" || raw === "system") theme = raw;
  } catch {
    // Standard bleibt.
  }
  applyTheme(theme);
}
