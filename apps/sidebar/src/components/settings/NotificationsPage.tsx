import { useEffect, useRef, useState } from "react";
import { Bell, Mail } from "lucide-react";
import {
  type AppSettings,
  type EmailConfig,
  type NotificationChannel,
  type NotificationSettings,
  type NotificationStatus,
  clearEmailConfig,
  getNotificationStatus,
  saveEmailConfig,
  setNotificationChannel,
  setNotificationTime,
  testNotification,
} from "@/lib/ipc";
import { SubPage } from "@/components/Shared";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
  AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

const EMPTY_EMAIL: EmailConfig = {
  host: "", port: 587, security: "startTls", username: "", from: "", to: "", hasPassword: false,
};
const SECURITY_LABELS = { startTls: "STARTTLS", tls: "TLS / SSL" };

type Props = {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
  onBack: () => void;
};

function EmailField({ label, id, ...props }: React.ComponentProps<typeof Input> & { label: string; id: string }) {
  return (
    <div className="space-y-2">
      <Label htmlFor={id}>{label}</Label>
      <Input id={id} className="h-7 rounded-md px-2 text-sm" {...props} />
    </div>
  );
}

export function NotificationsPage({ settings, onChange, onBack }: Props) {
  const notifications = settings.notifications;
  const [draft, setDraft] = useState<EmailConfig>(notifications.email ?? EMPTY_EMAIL);
  const [password, setPassword] = useState("");
  const [time, setTime] = useState(notifications.reminderTime);
  const [busy, setBusy] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<{ error: boolean; text: string } | null>(null);
  const [status, setStatus] = useState<NotificationStatus | null>(null);
  const [removeOpen, setRemoveOpen] = useState(false);
  const settingsRef = useRef(settings);
  settingsRef.current = settings;
  const busyRef = useRef(false);
  const dirty = password !== "" || JSON.stringify(draft) !== JSON.stringify(notifications.email ?? EMPTY_EMAIL);
  const canReusePassword = !!notifications.email?.hasPassword
    && draft.host.trim().toLowerCase() === notifications.email.host
    && draft.port === notifications.email.port
    && draft.username.trim() === notifications.email.username;

  useEffect(() => {
    let active = true;
    const refresh = () => void getNotificationStatus().then((value) => {
      if (active) setStatus(value);
    }).catch(() => {});
    refresh();
    const timer = window.setInterval(refresh, 30_000);
    return () => { active = false; window.clearInterval(timer); };
  }, []);

  useEffect(() => setTime(notifications.reminderTime), [notifications.reminderTime]);

  const apply = (next: NotificationSettings) => {
    onChange({ ...settingsRef.current, notifications: next });
  };

  const run = async (name: string, action: () => Promise<void>) => {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(name);
    setFeedback(null);
    try {
      await action();
    } catch (error) {
      setFeedback({ error: true, text: String(error) });
    } finally {
      busyRef.current = false;
      setBusy(null);
      void getNotificationStatus().then(setStatus).catch(() => {});
    }
  };

  const toggle = (channel: NotificationChannel, enabled: boolean) => {
    void run(channel, async () => apply(await setNotificationChannel(channel, enabled)));
  };
  const test = (channel: NotificationChannel) => {
    void run(`test-${channel}`, async () => {
      await testNotification(channel);
      setFeedback({ error: false, text: channel === "popup"
        ? "Test-Popup an das Betriebssystem übergeben. Anzeige bei Bedarf in den Systemeinstellungen erlauben."
        : "Test-E-Mail versendet. Bitte auch den Spam-Ordner prüfen." });
    });
  };
  const commitTime = () => {
    if (time === notifications.reminderTime) return;
    void run("time", async () => {
      try { apply(await setNotificationTime(time)); }
      catch (error) { setTime(notifications.reminderTime); throw error; }
    });
  };
  const edit = <K extends keyof EmailConfig>(key: K, value: EmailConfig[K]) =>
    setDraft((current) => ({ ...current, [key]: value }));

  return (
    <SubPage title="Benachrichtigungen" onBack={onBack} backDisabled={busy !== null}>
      <p className="text-xs leading-snug text-muted-foreground">
        Für offene Aufgaben mit Fälligkeitsdatum. Jeder Kanal lässt sich einzeln einschalten.
      </p>
      {feedback && <p role={feedback.error ? "alert" : "status"} aria-live="polite"
        className={`sticky top-0 z-10 break-words rounded-md border border-border bg-card p-2 text-xs leading-snug ${feedback.error ? "text-destructive" : "text-muted-foreground"}`}>
        {feedback.text}
      </p>}

      <section aria-labelledby="popup-label" className="space-y-2">
        <div className="flex items-center justify-between gap-2">
          <Label id="popup-label" htmlFor="popup-enabled">
            <Bell className="size-4 shrink-0 text-muted-foreground" />Native Popups
          </Label>
          <Switch id="popup-enabled" size="sm" checked={notifications.popupEnabled}
            disabled={busy !== null} onCheckedChange={(value) => toggle("popup", value)} />
        </div>
        <p className="text-xs leading-snug text-muted-foreground">
          Mitteilungen von macOS oder Windows, auch bei eingeklappter Sidebar.
          Anzeige und Ton steuerst du zusätzlich in den Systemeinstellungen.
        </p>
        {status?.popupError && <p role="alert" className="text-xs text-destructive">{status.popupError}</p>}
        <Button variant="outline" size="sm" className="w-full" disabled={busy !== null} onClick={() => test("popup")}>
          {busy === "test-popup" ? "Wird getestet…" : "Test-Popup anzeigen"}
        </Button>
      </section>

      <Separator />

      <section aria-labelledby="email-label" className="space-y-2">
        <div className="flex items-center justify-between gap-2">
          <Label id="email-label" htmlFor="email-enabled">
            <Mail className="size-4 shrink-0 text-muted-foreground" />E-Mail
          </Label>
          <Switch id="email-enabled" size="sm" checked={notifications.emailEnabled}
            disabled={busy !== null || (!notifications.emailEnabled && !notifications.email?.hasPassword)}
            onCheckedChange={(value) => toggle("email", value)} />
        </div>
        <p className="break-words text-xs leading-snug text-muted-foreground">
          {notifications.email ? `Empfänger: ${notifications.email.to}` : "Zuerst unten den E-Mail-Zugang einrichten und speichern."}
        </p>
        {status?.emailError && <p role="alert" className="text-xs text-destructive">{status.emailError}</p>}
        {status?.lastEmailSentAt && <p className="text-xs text-muted-foreground">
          Zuletzt versendet: {new Date(status.lastEmailSentAt).toLocaleString("de-DE")}
        </p>}
      </section>

      <Separator />

      <section aria-labelledby="reminder-time-label" className="space-y-2">
        <Label id="reminder-time-label" htmlFor="reminder-time">Erinnerungszeit</Label>
        <Input id="reminder-time" type="time" required value={time} disabled={busy !== null}
          className="h-7 rounded-md px-2 text-sm [color-scheme:light] dark:[color-scheme:dark]" onChange={(event) => setTime(event.target.value)}
          onBlur={commitTime} onKeyDown={(event) => { if (event.key === "Enter") event.currentTarget.blur(); }} />
        <p className="text-xs leading-snug text-muted-foreground">
          Am Fälligkeitstag zur lokalen Uhrzeit, einmal je Aufgabe und Kanal.
          Offene überfällige Aufgaben werden beim nächsten App-Start nachgeholt.
          Mehrere Aufgaben kommen als Zusammenfassung. Die App muss laufen.
        </p>
      </section>

      <Separator />

      <form className="space-y-4" onSubmit={(event) => {
        event.preventDefault();
        void run("save", async () => {
          const next = await saveEmailConfig(draft, password || null);
          apply(next);
          setDraft(next.email ?? EMPTY_EMAIL);
          setPassword("");
          setFeedback({ error: false, text: next.emailEnabled
            ? "E-Mail-Zugang gespeichert. E-Mail-Benachrichtigungen sind aktiv."
            : "E-Mail-Zugang gespeichert. Mit Test-E-Mail prüfen, dann E-Mail einschalten." });
        });
      }}>
        <div className="space-y-2">
          <h2 className="text-sm font-medium">E-Mail einrichten</h2>
          <p className="text-xs leading-snug text-muted-foreground">
            Nutze die SMTP-Daten deines Mail-Anbieters. Versand erfolgt verschlüsselt.
            Manche Anbieter benötigen ein App-Passwort.
          </p>
        </div>
        <fieldset disabled={busy !== null} className="min-w-0 space-y-4">
          <EmailField id="smtp-host" label="SMTP-Host" value={draft.host} required
            placeholder="smtp.example.com" autoCapitalize="none" spellCheck={false}
            onChange={(event) => edit("host", event.target.value)} />
          <div className="space-y-2">
            <Label htmlFor="smtp-security">Verschlüsselung</Label>
            <Select value={draft.security} disabled={busy !== null} onValueChange={(value) => {
              if (value !== "tls" && value !== "startTls") return;
              setDraft((current) => ({ ...current, security: value,
                port: [465, 587].includes(current.port) ? (value === "tls" ? 465 : 587) : current.port }));
            }}>
              <SelectTrigger id="smtp-security" size="sm" className="w-full">
                <SelectValue>{(value: string | null) => SECURITY_LABELS[value as keyof typeof SECURITY_LABELS]}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="startTls">STARTTLS</SelectItem>
                <SelectItem value="tls">TLS / SSL</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">Üblich: STARTTLS mit Port 587, TLS mit Port 465.</p>
          </div>
          <EmailField id="smtp-port" label="Port" type="number" min={1} max={65535} step={1}
            required value={draft.port || ""} onChange={(event) => edit("port", Number(event.target.value))} />
          <EmailField id="smtp-username" label="SMTP-Benutzername" required value={draft.username}
            autoComplete="username" autoCapitalize="none" spellCheck={false}
            onChange={(event) => edit("username", event.target.value)} />
          <div className="space-y-2">
            <EmailField id="smtp-password" label="Passwort / App-Passwort" type="password"
              autoComplete="new-password" value={password} onChange={(event) => setPassword(event.target.value)}
              required={!canReusePassword}
              placeholder={canReusePassword ? "Gespeichert · leer lassen zum Beibehalten" : "App-Passwort eingeben"} />
            <p className="text-xs leading-snug text-muted-foreground">
              Sicher im System-Schlüsselbund gespeichert. Bei neuem Host, Port oder Benutzernamen erneut eingeben.
            </p>
          </div>
          <EmailField id="smtp-from" label="Absenderadresse" type="email" required value={draft.from}
            placeholder="du@example.com" autoCapitalize="none" spellCheck={false}
            onChange={(event) => edit("from", event.target.value)} />
          <EmailField id="smtp-to" label="Empfängeradresse" type="email" required value={draft.to}
            placeholder="du@example.com" autoCapitalize="none" spellCheck={false}
            onChange={(event) => edit("to", event.target.value)} />
          <Button type="submit" size="sm" className="w-full" disabled={!dirty && !!notifications.email}>
            {busy === "save" ? "Wird gespeichert…" : "E-Mail-Zugang speichern"}
          </Button>
        </fieldset>
      </form>

      <div className="space-y-2">
        <Button variant="outline" size="sm" className="w-full"
          disabled={busy !== null || !notifications.email?.hasPassword || dirty} onClick={() => test("email")}>
          {busy === "test-email" ? "Wird versendet…" : "Test-E-Mail senden"}
        </Button>
        {dirty && <p className="text-xs text-muted-foreground">Änderungen vor dem Test speichern.</p>}
        {notifications.email && <AlertDialog open={removeOpen} onOpenChange={setRemoveOpen}>
          <AlertDialogTrigger render={<Button variant="ghost" size="sm" className="w-full text-muted-foreground" disabled={busy !== null} />}>
            E-Mail-Zugang entfernen
          </AlertDialogTrigger>
          <AlertDialogContent size="sm" className="rounded-lg">
            <AlertDialogHeader>
              <AlertDialogTitle className="text-sm">E-Mail-Zugang entfernen?</AlertDialogTitle>
              <AlertDialogDescription>
                E-Mail-Benachrichtigungen werden ausgeschaltet. Zugangsdaten und gespeichertes Passwort werden entfernt.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel size="sm" disabled={busy !== null}>Abbrechen</AlertDialogCancel>
              <AlertDialogAction size="sm" disabled={busy !== null} onClick={() => {
                void run("remove", async () => {
                  apply(await clearEmailConfig());
                  setDraft(EMPTY_EMAIL);
                  setPassword("");
                  setRemoveOpen(false);
                  setFeedback({ error: false, text: "E-Mail-Zugang entfernt." });
                });
              }}>Entfernen</AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>}
      </div>

    </SubPage>
  );
}
