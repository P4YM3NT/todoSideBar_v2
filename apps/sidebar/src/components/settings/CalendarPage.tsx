import { useEffect, useRef, useState } from "react";
import { CalendarDays, RefreshCw } from "lucide-react";
import {
  type AppSettings, type CalendarInfo, type CalendarSettings, type CalendarStatus,
  getCalendarStatus, listCalendars, requestCalendarAccess, setCalendarSync, syncCalendarsNow,
} from "@/lib/ipc";
import { SubPage } from "@/components/Shared";
import { GoogleCalendarSection } from "@/components/settings/GoogleCalendarSection";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";

/** Mindestens ein Anbieter liefert Kalender: Apple per Freigabe, Google per Verbindung. */
const hasProvider = (status: CalendarStatus) =>
  (status.supported && status.permission === "authorized") || status.google.account !== null;

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

export function CalendarPage({ settings, onChange, onBack }: Props) {
  const config = settings.calendarSync;
  const [status, setStatus] = useState<CalendarStatus | null>(null);
  const [calendars, setCalendars] = useState<CalendarInfo[]>([]);
  const [selected, setSelected] = useState(config.calendarIds);
  const [loaded, setLoaded] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<{ error: boolean; text: string } | null>(null);
  const busyRef = useRef(false);
  const settingsRef = useRef(settings);
  settingsRef.current = settings;
  const dirty = [...selected].sort().join("\n") !== [...config.calendarIds].sort().join("\n");
  const authorized = status?.permission === "authorized";
  const connected = status !== null && hasProvider(status);
  const missing = loaded ? config.calendarIds.filter((id) => !calendars.some((calendar) => calendar.id === id && calendar.writable)) : [];

  useEffect(() => {
    let active = true;
    let refreshing = false;
    const refresh = async () => {
      if (refreshing || busyRef.current) return;
      refreshing = true;
      try {
        const next = await getCalendarStatus();
        if (!active) return;
        setStatus(next);
        if (hasProvider(next) && !next.syncing) {
          const values = await listCalendars();
          if (active) { setCalendars(values); setLoaded(true); }
        } else if (!hasProvider(next)) {
          setLoaded(false); setCalendars([]);
        }
      } catch (error) {
        if (active) setFeedback({ error: true, text: String(error) });
      } finally {
        refreshing = false;
      }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), 10_000);
    return () => { active = false; window.clearInterval(timer); };
  }, []);

  useEffect(() => setSelected(config.calendarIds), [config.calendarIds]);

  const run = async (name: string, action: () => Promise<void>) => {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(name);
    setFeedback(null);
    try { await action(); }
    catch (error) { setFeedback({ error: true, text: String(error) }); }
    finally {
      busyRef.current = false;
      setBusy(null);
      void getCalendarStatus().then(setStatus).catch(() => {});
    }
  };

  const reloadCalendars = async () => {
    const next = await getCalendarStatus();
    setStatus(next);
    if (hasProvider(next)) { setCalendars(await listCalendars()); setLoaded(true); }
    else { setCalendars([]); setLoaded(false); }
  };

  const apply = async (next: CalendarSettings) => {
    const saved = await setCalendarSync(next);
    onChange({ ...settingsRef.current, calendarSync: saved });
    setSelected(saved.calendarIds);
  };

  return (
    <SubPage title="Kalendersync" onBack={onBack} backDisabled={busy !== null}>
      <p className="text-xs leading-snug text-muted-foreground">
        Aufgaben mit Datum als Termine in Apple- und Google-Kalendern. Notion Calendar zeigt
        verbundene Google- und iCloud-Kalender an, dort erscheinen die Aufgaben automatisch.
        Änderungen an Titel, Notizen, Datum und Uhrzeiten werden in beide Richtungen übernommen.
      </p>

      {feedback && <p role={feedback.error ? "alert" : "status"} aria-live="polite"
        className={`break-words rounded-md border border-border bg-card p-2 text-xs leading-snug ${feedback.error ? "text-destructive" : "text-muted-foreground"}`}>
        {feedback.text}
      </p>}

      {!status && <p role="status" className="text-xs text-muted-foreground">Kalenderstatus wird geladen…</p>}
      {status && <>
        {status.supported && <><section className="space-y-2" aria-labelledby="calendar-accounts">
          <h2 id="calendar-accounts" className="text-sm font-medium">Apple Kalender</h2>
          <p className="text-xs leading-snug text-muted-foreground">
            iCloud und Google zuerst unter macOS → Systemeinstellungen → Internetaccounts hinzufügen
            und „Kalender“ aktivieren. Bereits eingerichtete Konten erscheinen nach der Freigabe hier.
          </p>
          {authorized ? <p className="text-xs text-muted-foreground">Kalenderzugriff erlaubt.</p> : <>
            <Button size="sm" className="w-full" disabled={busy !== null || status.permission === "restricted"}
              onClick={() => void run("access", async () => {
                const next = await requestCalendarAccess();
                setStatus(next);
                if (next.permission === "authorized") {
                  setCalendars(await listCalendars());
                  setLoaded(true);
                } else {
                  setFeedback({ error: true, text: "Kalenderzugriff unter Systemeinstellungen → Datenschutz & Sicherheit → Kalender erlauben." });
                }
              })}>
              {busy === "access" ? "Warte auf Freigabe…" : "Kalenderzugriff erlauben"}
            </Button>
            {status.permission !== "notDetermined" && <p className="text-xs leading-snug text-muted-foreground">
              {status.permission === "restricted" ? "Kalenderzugriff ist durch macOS eingeschränkt."
                : "Voller Kalenderzugriff nötig. Freigabe unter Datenschutz & Sicherheit → Kalender prüfen."}
            </p>}
          </>}
        </section>

        <Separator /></>}

        <GoogleCalendarSection status={status} busy={busy} run={run} onStatus={setStatus}
          onConnectionChanged={reloadCalendars} />

        <Separator />

        <section className="space-y-2" aria-labelledby="calendar-targets">
          <h2 id="calendar-targets" className="text-sm font-medium">Zielkalender</h2>
          <p className="text-xs leading-snug text-muted-foreground">
            Jede datierte Aufgabe erscheint in allen ausgewählten Kalendern.
            Andere Kalendertermine bleiben unverändert.
          </p>
          {connected && loaded && calendars.length === 0 && <p className="text-xs text-muted-foreground">
            Keine Kalender gefunden. Konten prüfen, dann Liste aktualisieren.
          </p>}
          <div className="space-y-2">
            {calendars.map((calendar, index) => <div key={calendar.id} className="flex items-start gap-2 rounded-md border border-border p-2">
              <Checkbox id={`calendar-${index}`} checked={selected.includes(calendar.id)}
                disabled={busy !== null || (!calendar.writable && !selected.includes(calendar.id))}
                onCheckedChange={(checked) => setSelected((current) => checked
                  ? [...new Set([...current, calendar.id])]
                  : current.filter((id) => id !== calendar.id))} />
              <Label htmlFor={`calendar-${index}`} className="min-w-0 flex-1 flex-col items-start gap-1">
                <span className="block w-full truncate text-sm">{calendar.title}</span>
                <span className="block w-full truncate text-xs font-normal text-muted-foreground">
                  {calendar.account}{!calendar.writable ? " · Schreibgeschützt" : ""}
                </span>
              </Label>
            </div>)}
          </div>
          {missing.length > 0 && <p role="alert" className="text-xs leading-snug text-destructive">
            {missing.length} gespeicherte Kalender fehlen oder sind schreibgeschützt. Auswahl prüfen und speichern.
          </p>}
          {missing.length > 0 && <Button variant="outline" size="sm" className="w-full" disabled={busy !== null}
            onClick={() => setSelected((current) => current.filter((id) => calendars.some((calendar) => calendar.id === id && calendar.writable)))}>
            Nicht verfügbare Kalender abwählen
          </Button>}
          {!connected && <p className="text-xs text-muted-foreground">
            Zuerst Apple-Kalenderzugriff erlauben oder Google verbinden.
          </p>}
          {connected && <>
            <Button variant="outline" size="sm" className="w-full" disabled={busy !== null}
              onClick={() => void run("refresh", async () => {
                setCalendars(await listCalendars()); setLoaded(true);
              })}>
              <RefreshCw className="size-4" />Kalender aktualisieren
            </Button>
            <Button size="sm" className="w-full" disabled={busy !== null || !dirty || (config.enabled && selected.length === 0)}
              onClick={() => void run("save", async () => {
                const valid = selected.filter((id) => calendars.some((calendar) => calendar.id === id && calendar.writable));
                await apply({ enabled: config.enabled, calendarIds: valid });
                setFeedback({ error: false, text: "Kalenderauswahl gespeichert." });
              })}>
              {busy === "save" ? "Wird gespeichert…" : "Auswahl speichern"}
            </Button>
          </>}
          <p className="text-xs leading-snug text-muted-foreground">
            Abgewählte Kalender behalten vorhandene Termine. Deren Synchronisierung pausiert.
          </p>
        </section>

        <Separator />

        <section className="space-y-2" aria-labelledby="calendar-auto-label">
          <div className="flex items-center justify-between gap-2">
            <Label id="calendar-auto-label" htmlFor="calendar-auto">
              <CalendarDays className="size-4 shrink-0 text-muted-foreground" />Automatischer Sync
            </Label>
            <Switch id="calendar-auto" size="sm" checked={config.enabled}
              disabled={busy !== null || (!config.enabled && (!connected || config.calendarIds.length === 0 || dirty || missing.length > 0))}
              onCheckedChange={(enabled) => void run("toggle", () => apply({ ...config, enabled }))} />
          </div>
          <p className="text-xs leading-snug text-muted-foreground">
            Bei Aufgabenänderungen und jede Minute, solange die App läuft. Bei Apple-Kalendern
            übernimmt macOS den Abgleich mit iCloud; dessen Aktualisierungsintervall kann zusätzliche
            Verzögerungen verursachen. Direkt verbundene Google-Kalender werden sofort geschrieben.
          </p>
          {dirty && <p className="text-xs text-muted-foreground">Neue Auswahl zuerst speichern.</p>}
          {status.error && <p role="alert" className="break-words text-xs leading-snug text-destructive">{status.error}</p>}
          {status.warnings?.length > 0 && <div role="status" className="space-y-2 text-xs leading-snug">
            <p className="text-muted-foreground">Andere Aufgaben werden weiter synchronisiert.</p>
            {status.warnings.map((warning) => <p key={warning} className="break-words text-destructive">{warning}</p>)}
          </div>}
          {status.lastSyncedAt && <p className="text-xs leading-snug text-muted-foreground">
            Letzter Sync: {new Date(status.lastSyncedAt).toLocaleString("de-DE")} · {status.syncedTasks} Aufgaben
          </p>}
          <Button variant="outline" size="sm" className="w-full"
            disabled={busy !== null || status.syncing || !config.enabled || !connected || dirty}
            onClick={() => void run("sync", async () => {
              const next = await syncCalendarsNow();
              setStatus(next);
              setFeedback({ error: false, text: next.warnings.length ? "Sync abgeschlossen. Hinweise zu übersprungenen Aufgaben beachten." : "Aufgaben und Kalender synchronisiert." });
            })}>
            <RefreshCw className={`size-4 ${status.syncing || busy === "sync" ? "animate-spin" : ""}`} />
            {status.syncing || busy === "sync" ? "Synchronisiert…" : "Jetzt synchronisieren"}
          </Button>
          <p className="text-xs leading-snug text-muted-foreground">
            Geplantes Datum hat Vorrang vor Fälligkeit. Bei Konflikten gewinnt die neuere Änderung.
            Erledigte, gelöschte oder undatierte Aufgaben verlieren ihre Kalenderkopien.
            Im Kalender gelöschte Termine bleiben als Aufgaben erhalten; Änderungen an Titel, Notizen, Datum oder Uhrzeiten legen sie neu an.
            Von/Bis und Enddatum werden in beide Richtungen übernommen. Ohne Uhrzeiten bleibt der Termin ganztägig.
            Wiederholende Termine werden übersprungen.
          </p>
        </section>
      </>}
    </SubPage>
  );
}
