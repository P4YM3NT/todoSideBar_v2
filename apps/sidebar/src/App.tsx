import { useEffect, useRef, useState } from "react";
import { Handle } from "@/components/Handle";
import { ResizeGrip } from "@/components/ResizeGrip";
import { Settings } from "@/components/Settings";
import { Sidebar } from "@/components/Sidebar";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  type AppSettings,
  HANDLE_W,
  commitSidebarWidth,
  getSettings,
  onShowSettings,
  setFilterPresets,
  toggleSidebar,
} from "@/lib/ipc";
import { isDarkColor, readableOn } from "@/lib/accent";
import { useDetailBackground } from "@/lib/useDetailBackground";
import { cn } from "@/lib/utils";
import { loadLastView, saveLastView } from "@/lib/lastView";
import { type Theme, applyTheme } from "@/lib/theme";
import { setTheme } from "@/lib/ipc";
import type { View } from "@/lib/tasks";
import { useSidebarPhase } from "@/lib/useSidebarPhase";

export function App() {
  const { phase, onTransitionEnd } = useSidebarPhase();
  // Haelt die Fensterfarbe fuer das Detailfenster aktuell, auch bevor es zum
  // ersten Mal geoeffnet wird.
  useDetailBackground();
  const [view, setView] = useState<"list" | "settings">("list");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  // Liegt hier statt in `Sidebar`: die wird beim Wechsel in die Einstellungen
  // ausgehaengt und verlaere ihren Zustand.
  const [listView, setListView] = useState<View>(loadLastView);

  // Die Fensterkante wird von macOS selbst gezogen - der WebView bekommt davon
  // nichts mit. Ohne diesen Horcher bliebe die neue Breite unbekannt und
  // `apply_frame` wuerde beim naechsten Aufklappen auf den alten Wert
  // zuruecksetzen.
  const widthRef = useRef(0);
  widthRef.current = settings?.width ?? 0;

  useEffect(() => {
    if (phase !== "expanded") return;
    let timer: ReturnType<typeof setTimeout>;

    // Skalierung einmal holen statt bei jedem Ereignis - waehrend des Ziehens
    // feuert das dutzendfach pro Sekunde.
    let scale = 1;
    let cancelled = false;
    void getCurrentWindow()
      .scaleFactor()
      .then((s) => {
        if (!cancelled) scale = s;
      });

    const un = getCurrentWindow().onResized(({ payload }) => {
      const logical = payload.width / scale;
      // Eigene Groessenwechsel nicht zurueckschreiben - sonst schaukelt sich
      // Setzen und Speichern gegenseitig hoch.
      if (Math.abs(logical - widthRef.current) < 1) return;

      // Anzeige sofort mitziehen, damit das Eingabefeld live folgt.
      setSettings((prev) => (prev ? { ...prev, width: logical } : prev));

      // Gespeichert wird nur einmal, wenn das Ziehen zur Ruhe kommt.
      clearTimeout(timer);
      timer = setTimeout(() => void commitSidebarWidth(logical), 250);
    });

    return () => {
      cancelled = true;
      clearTimeout(timer);
      void un.then((f) => f());
    };
  }, [phase]);

  const changeTheme = (theme: Theme) => {
    applyTheme(theme);
    void setTheme(theme);
    setSettings((prev) => (prev ? { ...prev, theme } : prev));
  };

  const changeView = (next: View) => {
    setListView(next);
    saveLastView(next);
  };

  useEffect(() => {
    void getSettings().then((s) => {
      setSettings(s);
      // Der Spiegel in localStorage hat beim Start schon etwas gesetzt; hier
      // wird auf den echten Stand aus der Datenbank korrigiert.
      applyTheme(s.theme);
    });
  }, []);
  const open = phase === "expanded";
  const animating = phase === "expanded" || phase === "closing";
  // Vor dem Laden von true ausgehen - das ist der Standard und vermeidet,
  // dass der Griff kurz aufblitzt.
  const showHandle = settings?.handleVisible ?? true;

  useEffect(() => {
    const sub = onShowSettings(() => setView("settings"));
    return () => void sub.then((un) => un());
  }, []);

  return (
    <div
      onTransitionEnd={onTransitionEnd}
      className="flex h-full will-change-transform"
      style={{
        // Ohne Griff muss der Inhalt vollstaendig hinausfahren - sonst bliebe
        // ein 14 px breiter Streifen der Sidebar stehen.
        transform: open
          ? "translateX(0)"
          : showHandle
            ? `translateX(calc(100% - ${HANDLE_W}px))`
            : "translateX(100%)",
        // Nur transform animieren - Breite oder Position zu animieren wuerde
        // bei jedem Frame das Layout neu berechnen.
        transition: animating
          ? "transform 200ms cubic-bezier(0.32, 0.72, 0, 1)"
          : "none",
      }}
    >
      {showHandle && (
        <Handle expanded={open} onToggle={() => void toggleSidebar()} />
      )}

      <aside
        className={cn(
          "relative h-full flex-1 overflow-hidden text-foreground backdrop-blur-xl",
          // Eine eigene Flaechenfarbe ueberschreibt nur den Hintergrund - Schrift,
          // Rahmen und Muted-Toene kaemen weiter vom globalen Theme und waeren
          // womoeglich unlesbar. Deshalb entscheidet die Helligkeit der Farbe,
          // welcher Token-Satz in der Sidebar gilt.
          settings?.backgroundColor &&
            (isDarkColor(settings.backgroundColor) ? "dark" : "light"),
        )}
        // Nutzerfarbe kommt als Variable, nicht als Klasse - genau wie
        // Projektfarben. Ohne gesetzten Wert greifen die Theme-Tokens.
        style={
          {
            // Immer gesetzt: ohne eigene Wahl faellt es auf die Themefarbe
            // zurueck, damit `--checkbox-accent` ueberall verwendbar bleibt.
            "--checkbox-accent": settings?.checkboxColor ?? "var(--primary)",
            "--checkbox-accent-fg": settings?.checkboxColor
              ? readableOn(settings.checkboxColor)
              : "var(--primary-foreground)",
            // Deckkraft als color-mix statt Tailwind-Opacity: der Wert ist frei
            // einstellbar, Tailwind kennt nur feste Stufen.
            backgroundColor: `color-mix(in oklab, ${
              settings?.backgroundColor ?? "var(--card)"
            } ${settings?.backgroundAlpha ?? 95}%, transparent)`,
          } as React.CSSProperties
        }
      >
        <ResizeGrip />
        <div className="h-full p-3">
          {view === "settings" ? (
            settings && (
              <Settings
                settings={settings}
                onClose={() => setView("list")}
                onChange={setSettings}
              />
            )
          ) : (
            <Sidebar
              view={listView}
              priorities={settings?.priorities ?? []}
              showSubtasks={settings?.showSubtasks ?? false}
              theme={settings?.theme ?? "light"}
              addHotkey={settings?.addHotkey ?? null}
              presets={settings?.filterPresets ?? []}
              onPresetsChange={(filterPresets) => {
                void setFilterPresets(filterPresets);
                setSettings((prev) => (prev ? { ...prev, filterPresets } : prev));
              }}
              onThemeChange={changeTheme}
              onViewChange={changeView}
              onOpenSettings={() => setView("settings")}
            />
          )}
        </div>
      </aside>
    </div>
  );
}
