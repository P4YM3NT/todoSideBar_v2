import type { View } from "./tasks";

const KEY = "todosidebar.lastView";
const FALLBACK: View = { kind: "today" };

/**
 * Zuletzt gewaehlte Ansicht. Liegt in localStorage und nicht in der Datenbank:
 * das ist reiner Oberflaechenzustand dieses Fensters, kein geteiltes Datum.
 *
 * Lesen und Schreiben sind abgesichert - in manchen Kontexten wirft der Zugriff
 * selbst (privates Fenster, geleerte Site-Daten).
 */
export function loadLastView(): View {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return FALLBACK;
    const parsed = JSON.parse(raw) as View;
    return parsed?.kind ? parsed : FALLBACK;
  } catch {
    return FALLBACK;
  }
}

export function saveLastView(view: View) {
  try {
    localStorage.setItem(KEY, JSON.stringify(view));
  } catch {
    // Nicht speichern zu koennen darf die Bedienung nicht stoeren.
  }
}

export { FALLBACK as DEFAULT_VIEW };
