import { useEffect, useState } from "react";
import { LogIn, LogOut } from "lucide-react";
import {
  type CalendarStatus, connectGoogleCalendar, disconnectGoogleCalendar,
} from "@/lib/ipc";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

type Props = {
  status: CalendarStatus;
  busy: string | null;
  /** Serialisierte Aktion der Kalenderseite inkl. Fehleranzeige. */
  run: (name: string, action: () => Promise<void>) => Promise<void>;
  onStatus: (status: CalendarStatus) => void;
  onConnectionChanged: () => Promise<void>;
};

/** Direkte Verbindung zur Google Calendar API (OAuth im System-Browser). */
export function GoogleCalendarSection({ status, busy, run, onStatus, onConnectionChanged }: Props) {
  const { account, clientId: savedClientId } = status.google;
  const [clientId, setClientId] = useState(savedClientId ?? "");
  const [clientSecret, setClientSecret] = useState("");
  useEffect(() => { if (savedClientId) setClientId(savedClientId); }, [savedClientId]);

  if (account) {
    return (
      <section className="space-y-2" aria-labelledby="google-calendar">
        <h2 id="google-calendar" className="text-sm font-medium">Google Kalender</h2>
        <p className="break-words text-xs text-muted-foreground">Verbunden als {account}.</p>
        <Button variant="outline" size="sm" className="w-full" disabled={busy !== null}
          onClick={() => void run("google-disconnect", async () => {
            onStatus(await disconnectGoogleCalendar());
            await onConnectionChanged();
          })}>
          <LogOut className="size-4" />{busy === "google-disconnect" ? "Wird getrennt…" : "Google trennen"}
        </Button>
        <p className="text-xs leading-snug text-muted-foreground">
          Trennen widerruft den Zugriff. Termine in Google bleiben erhalten.
        </p>
      </section>
    );
  }

  return (
    <section className="space-y-2" aria-labelledby="google-calendar">
      <h2 id="google-calendar" className="text-sm font-medium">Google Kalender</h2>
      <p className="text-xs leading-snug text-muted-foreground">
        Direkt über die Google-API, ohne macOS-Internetaccount. In der Google Cloud Console
        die Google Calendar API aktivieren und einen OAuth-Client vom Typ „Desktop-App“ anlegen.
        Zugangsdaten bleiben im Schlüsselbund.
      </p>
      <div className="space-y-2">
        <Label htmlFor="google-client-id">Client-ID</Label>
        <Input id="google-client-id" className="h-7 rounded-md px-2 text-sm" autoComplete="off"
          spellCheck={false} placeholder="….apps.googleusercontent.com" value={clientId}
          disabled={busy !== null} onChange={(event) => setClientId(event.target.value)} />
      </div>
      <div className="space-y-2">
        <Label htmlFor="google-client-secret">Clientschlüssel</Label>
        <Input id="google-client-secret" type="password" className="h-7 rounded-md px-2 text-sm"
          autoComplete="off" value={clientSecret} disabled={busy !== null}
          onChange={(event) => setClientSecret(event.target.value)} />
      </div>
      <Button size="sm" className="w-full"
        disabled={busy !== null || !clientId.trim() || !clientSecret.trim()}
        onClick={() => void run("google-connect", async () => {
          onStatus(await connectGoogleCalendar(clientId, clientSecret));
          setClientSecret("");
          await onConnectionChanged();
        })}>
        <LogIn className="size-4" />
        {busy === "google-connect" ? "Anmeldung im Browser abschließen…" : "Mit Google verbinden"}
      </Button>
    </section>
  );
}
