import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Theme } from "./theme";

export const HANDLE_W = 14;

export type SidebarState = {
  expanded: boolean;
  handleVisible: boolean;
  width: number;
};

type RawState = {
  expanded: boolean;
  handle_visible: boolean;
  width: number;
};

const fromRaw = (r: RawState): SidebarState => ({
  expanded: r.expanded,
  handleVisible: r.handle_visible,
  width: r.width,
});

export const getSidebarState = async (): Promise<SidebarState> =>
  fromRaw(await invoke<RawState>("sidebar_state"));

export const expandSidebar = () => invoke<void>("sidebar_expand");
export const collapseSidebar = () => invoke<void>("sidebar_collapse");
export const toggleSidebar = () => invoke<void>("sidebar_toggle");
/** Nur waehrend des Ziehens - speichert bewusst nicht. */
export const setSidebarWidth = (width: number) =>
  invoke<void>("sidebar_set_width", { width });

/** Endgueltige Breite: setzt und speichert. Liefert den geklemmten Wert. */
export const commitSidebarWidth = (width: number) =>
  invoke<number>("sidebar_commit_width", { width });

/** Setzt auf den Standardanteil des Monitors zurück und liefert die neue Breite. */
export const resetSidebarWidth = () => invoke<number>("reset_sidebar_width");
export const setHandleVisible = (visible: boolean) =>
  invoke<void>("sidebar_set_handle_visible", { visible });

export type AppSettings = {
  handleVisible: boolean;
  hotkey: string;
  /** Öffnet die Sidebar und legt sofort eine Aufgabe an. */
  addHotkey: string;
  width: number;
  /** Hex-Farbe für abgehakte Checkboxen. `null` = Themefarbe. */
  checkboxColor: string | null;
  theme: Theme;
  /** Deckkraft der Sidebar-Fläche in Prozent. */
  backgroundAlpha: number;
  /** Eigene Hintergrundfarbe (beliebige CSS-Farbe). `null` = Themefarbe. */
  backgroundColor: string | null;
  priorities: Priority[];
  /** Unteraufgaben in den festen Ansichten als eigene Zeilen zeigen. */
  showSubtasks: boolean;
};

/**
 * `id` ist stabil und wird nie neu vergeben - `task.priority` zeigt darauf.
 * Die Reihenfolge in der Liste bestimmt den Rang.
 */
export type Priority = {
  id: number;
  label: string;
  color: string;
};

export const getSettings = () => invoke<AppSettings>("get_settings");

export const setTheme = (theme: Theme) => invoke<void>("set_theme", { theme });

export const setBackgroundAlpha = (alpha: number) =>
  invoke<void>("set_background_alpha", { alpha });

export const setBackgroundColor = (color: string | null) =>
  invoke<void>("set_background_color", { color });

/** Entfernte Stufen werden dabei aus allen Aufgaben geräumt. */
export const setPriorities = (priorities: Priority[]) =>
  invoke<void>("set_priorities", { priorities });

export const setShowSubtasks = (show: boolean) =>
  invoke<void>("set_show_subtasks", { show });

/** `null` setzt auf die Themefarbe zurück. */
export const setCheckboxColor = (color: string | null) =>
  invoke<void>("set_checkbox_color", { color });

/** Wirft, wenn die Kombination bereits belegt ist - dann bleibt der alte Hotkey aktiv. */
export const setHotkey = (accelerator: string) =>
  invoke<void>("set_hotkey", { accelerator });

/** Wie `setHotkey`, nur für den Hotkey zum Anlegen. */
export const setAddHotkey = (accelerator: string) =>
  invoke<void>("set_add_hotkey", { accelerator });

/** Der Hotkey zum Anlegen wurde gedrückt - die Sidebar steht bereits offen. */
export const onQuickAdd = (cb: () => void) =>
  listen("sidebar://quick-add", () => cb());

/**
 * Das Theme wurde geändert. Braucht das Detailfenster: es wird beim Schließen
 * nur versteckt und bekäme den Wechsel sonst nicht mit.
 */
export const onThemeChanged = (cb: (theme: Theme) => void) =>
  listen<Theme>("theme://changed", (e) => cb(e.payload));

/** Tray-Menüeintrag "Einstellungen…". */
export const onShowSettings = (cb: () => void) =>
  listen("sidebar://show-settings", () => cb());

/** Rust hat den Rahmen bereits vergrößert - jetzt darf hineinanimiert werden. */
export const onOpen = (cb: (s: SidebarState) => void) =>
  listen<RawState>("sidebar://open", (e) => cb(fromRaw(e.payload)));

/** Aufforderung auszublenden. Erst danach darf der Rahmen schrumpfen. */
export const onCloseRequested = (cb: () => void) =>
  listen("sidebar://close", () => cb());

export const onClosed = (cb: (s: SidebarState) => void) =>
  listen<RawState>("sidebar://closed", (e) => cb(fromRaw(e.payload)));
