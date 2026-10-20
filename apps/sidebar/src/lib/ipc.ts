import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Theme } from "./theme";
import type { FilterPreset } from "./filters";

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
  filterPresets: FilterPreset[];
  /** Unteraufgaben in den festen Ansichten als eigene Zeilen zeigen. */
  showSubtasks: boolean;
  notifications: NotificationSettings;
  calendarSync: CalendarSettings;
};

export type CalendarSettings = { enabled: boolean; calendarIds: string[] };
export type CalendarProvider = "apple" | "google";
export type CalendarInfo = {
  id: string;
  title: string;
  account: string;
  writable: boolean;
  provider: CalendarProvider;
};
/** Nicht geheime Google-Verbindungsdaten. Tokens liegen im Schlüsselbund. */
export type GoogleConnection = { clientId: string | null; account: string | null };
export type CalendarStatus = {
  /** EventKit (Apple) verfügbar, also macOS. Google geht überall. */
  supported: boolean;
  permission: "notDetermined" | "authorized" | "writeOnly" | "denied" | "restricted";
  syncing: boolean;
  lastSyncedAt: string | null;
  syncedTasks: number;
  warnings: string[];
  error: string | null;
  google: GoogleConnection;
};
export const getCalendarStatus = () => invoke<CalendarStatus>("calendar_status");
export const requestCalendarAccess = () => invoke<CalendarStatus>("request_calendar_access");
export const listCalendars = () => invoke<CalendarInfo[]>("list_calendars");
export const setCalendarSync = (config: CalendarSettings) =>
  invoke<CalendarSettings>("set_calendar_sync", { config });
export const syncCalendarsNow = () => invoke<CalendarStatus>("sync_calendars_now");
/** Öffnet die Google-Anmeldung im Browser und wartet auf deren Abschluss. */
export const connectGoogleCalendar = (clientId: string, clientSecret: string) =>
  invoke<CalendarStatus>("google_connect", { clientId, clientSecret });
export const disconnectGoogleCalendar = () => invoke<CalendarStatus>("google_disconnect");

export type NotificationChannel = "popup" | "email";
export type EmailConfig = {
  host: string;
  port: number;
  security: "startTls" | "tls";
  username: string;
  from: string;
  to: string;
  hasPassword: boolean;
};
export type NotificationSettings = {
  popupEnabled: boolean;
  emailEnabled: boolean;
  reminderTime: string;
  email: EmailConfig | null;
};
export type NotificationStatus = {
  popupError: string | null;
  emailError: string | null;
  lastEmailSentAt: string | null;
};

export const setNotificationChannel = (channel: NotificationChannel, enabled: boolean) =>
  invoke<NotificationSettings>("set_notification_channel", { channel, enabled });
export const setNotificationTime = (time: string) =>
  invoke<NotificationSettings>("set_notification_time", { time });
export const saveEmailConfig = (config: EmailConfig, password: string | null) =>
  invoke<NotificationSettings>("save_email_config", { config, password });
export const clearEmailConfig = () => invoke<NotificationSettings>("clear_email_config");
export const testNotification = (channel: NotificationChannel) =>
  invoke<void>("test_notification", { channel });
export const getNotificationStatus = () => invoke<NotificationStatus>("notification_status");

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

export const setFilterPresets = (presets: FilterPreset[]) =>
  invoke<void>("set_filter_presets", { presets });

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
