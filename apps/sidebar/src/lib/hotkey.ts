/** Tastenkombinationen: Aufnahme und Darstellung an einer Stelle. */

const IS_MAC = navigator.platform.toLowerCase().includes("mac");

/** Reine Modifier sind als Hotkey nutzlos - sie sollen die Aufnahme nicht beenden. */
const MODIFIERS = new Set(["Control", "Shift", "Alt", "Meta"]);

/** Ein Tastendruck als Tauri-Accelerator, oder `null`, wenn er nicht taugt. */
export function toAccelerator(e: React.KeyboardEvent): string | null {
  if (MODIFIERS.has(e.key)) return null;

  const parts: string[] = [];
  if (e.metaKey) parts.push(IS_MAC ? "Command" : "Super");
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");

  // Ohne Modifier würde die Kombination global jede normale Eingabe abfangen.
  if (parts.length === 0) return null;

  const key =
    e.key.length === 1 ? e.key.toUpperCase() : e.key === " " ? "Space" : e.key;
  parts.push(key);
  return parts.join("+");
}

/**
 * Accelerator in die Schreibweise, die man auf der Tastatur sieht.
 *
 * Liegt hier statt im `HotkeyRecorder`, weil auch Tooltips den Hotkey zeigen -
 * etwa der über "Aufgabe hinzufügen".
 */
export const pretty = (a: string) =>
  IS_MAC
    ? a
        .replace("CmdOrCtrl", "⌘")
        .replace("Command", "⌘")
        .replace("Control", "⌃")
        .replace("Alt", "⌥")
        .replace("Shift", "⇧")
        .replaceAll("+", " ")
    : a;
