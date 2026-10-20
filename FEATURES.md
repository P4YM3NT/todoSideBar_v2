# Funktionen

Zurück zur [Übersicht und Installation](README.md).

---

## Die Sidebar

Ein einziges Fenster wechselt zwischen zwei Rahmen: eingeklappt ist es 14 px
breit und deckt nur den Griff am rechten Bildschirmrand ab, ausgeklappt nimmt es
die volle Höhe der Arbeitsfläche ein. Klicks neben dem Griff gehen im
eingeklappten Zustand an die App darunter — die Sidebar liegt zwar über allem,
steht aber nirgends im Weg.

- **Griff am Rand.** Ein Klick klappt auf und zu. Wer ihn nicht mag, blendet ihn
  in den Einstellungen aus und erreicht die Sidebar weiterhin über den Hotkey
  und das Symbol in der Menüleiste.
- **Breite frei einstellbar.** Ziehen an der linken Kante oder ein Zahlenwert in
  den Einstellungen. Standard sind 15 % der Bildschirmbreite, Minimum 220 px.
- **Kein Dock-Symbol, kein Alt-Tab.** Die App läuft als Hintergrundwerkzeug und
  stiehlt beim Aufklappen keinen Fokus — man kann weitertippen, während sie
  offen steht.
- **Menüleisten-Symbol** mit „Sidebar anzeigen", „Einstellungen…" und „Beenden".

## Ansichten

| Ansicht | Inhalt |
|---|---|
| **Inbox** | Alles ohne Projekt |
| **Heute** | Was für heute geplant oder fällig ist |
| **Demnächst** | Spätere Termine |
| **Erledigt** | Rückschau; hier lässt sich bewusst nichts anlegen |
| **Projekte** | Eine Ansicht je Projekt |

Die zuletzt gewählte Ansicht wird gemerkt. Zeigt sie auf ein inzwischen
gelöschtes Projekt, fällt sie auf „Heute" zurück.

## Aufgaben

**Schnell anlegen.** Eine Zeile unter der Liste klappt auf, man tippt und drückt
Enter. Die Zeile bleibt offen und leert sich — mehrere Aufgaben hintereinander
sind der Normalfall. Das Projekt wird aus der aktuellen Ansicht vorbelegt,
lässt sich aber überschreiben.

**Ausführlich anlegen und bearbeiten.** Ein zweites Fenster mit Platz für alles
Weitere: Beschreibung, Notizen, Projekt, geplantes Datum, Fälligkeitsdatum,
Priorität und Zeitschätzung. Aus der Schnellzeile führt `⌘↵` direkt dorthin und
nimmt das bereits Getippte mit.

**Unteraufgaben.** Jede Aufgabe kann eigene Unteraufgaben haben. Ob sie in den
festen Ansichten als eigene Zeilen erscheinen, ist eine globale Einstellung;
einzelne Projekte dürfen sie überschreiben. Öffnet man eine Unteraufgabe, führt
eine Zeile am Kopf zurück zur Hauptaufgabe.

**Abhaken.** Die erledigte Zeile gleitet aus der Liste. In den Suchergebnissen
bleibt sie bewusst stehen — man hat sie ja gerade gesucht.

**Duplizieren und Löschen** über das Kontextmenü an der Zeile, Löschen mit
Rückfrage.

**Suche** über `⌘F`. Durchsucht Titel, Beschreibung und Notizen über alle
Ansichten hinweg.

## Kalender synchronisieren (Apple, Google, Notion Calendar)

Unter **Einstellungen → Kalendersync** lassen sich datierte Aufgaben als
Ganztagstermine oder Termine mit festen Uhrzeiten in mehreren Apple- und Google-Kalendern spiegeln.
Titel, Notizen, Datum und Von/Bis werden in beide Richtungen synchronisiert. Das geplante
Datum hat Vorrang; fehlt es, wird das Fälligkeitsdatum verwendet. Beschreibung,
Projekt, Priorität und Zeitschätzung bleiben Aufgabenfelder. Unabhängige Termine
werden nicht als Aufgaben importiert und nicht verändert.

**Uhrzeiten:** Im Detailfenster ein geplantes oder fälliges Datum wählen und
„Ganztägig“ anklicken, um Uhrzeiten zu aktivieren. Von und Bis sind beide nötig;
Bis muss nach Von liegen. Die kleinen Chips stehen auch bei neuen Aufgaben in
derselben Metadatenzeile wie „Geplant“ und „Fällig“. Klick auf „Von“ oder „Bis“
öffnet die Auswahl für Stunden und Minuten; direkte Eingabe im Popover ist ebenfalls möglich.
Für Termine über Mitternacht oder mehrere Tage das Enddatum setzen. Bei
Ganztagsterminen ist es der letzte eingeschlossene Tag. Uhrzeiten erscheinen in
der lokalen Zeitzone und werden als absolute Zeitpunkte gespeichert. Beim
Verschieben des Datums bleiben lokale Uhrzeiten und Tagesabstand erhalten.
Nicht existierende Uhrzeiten während der Zeitumstellung werden abgelehnt.

**Apple Kalender (macOS):** iCloud (oder andere Konten) unter **macOS →
Systemeinstellungen → Internetaccounts** hinzufügen und Kalender aktivieren. In
der App „Kalenderzugriff erlauben“. Die App nutzt EventKit und die vorhandenen
macOS-Konten; dafür werden keine Passwörter oder Tokens gespeichert.

**Google Kalender (direkt, alle Plattformen):** Einmalig einen eigenen OAuth-Client anlegen:

1. In der [Google Cloud Console](https://console.cloud.google.com/) ein Projekt
   anlegen und unter „APIs & Dienste → Bibliothek“ die **Google Calendar API** aktivieren.
2. Unter „OAuth-Zustimmungsbildschirm“ Typ „Extern“ wählen und die eigene
   Google-Adresse als Testnutzer eintragen.
3. Unter „Anmeldedaten → Anmeldedaten erstellen → OAuth-Client-ID“ den Typ
   **Desktop-App** wählen.
4. Client-ID und Clientschlüssel in **Einstellungen → Kalendersync → Google
   Kalender** eintragen und „Mit Google verbinden“ klicken. Die Anmeldung läuft im
   System-Browser (OAuth mit PKCE und Loopback-Weiterleitung). Beide Berechtigungen
   (Kalenderliste lesen, Termine bearbeiten) bestätigen.

Refresh-Token und Clientschlüssel liegen im Schlüsselbund, nie in der Datenbank.
„Google trennen“ widerruft den Zugriff; vorhandene Termine bleiben. Google-Kalender,
die zusätzlich als macOS-Internetaccount eingerichtet sind, erscheinen doppelt –
pro Kalender nur einen der beiden Wege auswählen, sonst entstehen doppelte Termine.
Solange der OAuth-Zustimmungsbildschirm im Testmodus ist, läuft das Refresh-Token
laut Google nach sieben Tagen ab. Für Dauerbetrieb den Status auf „In Produktion“
setzen; ohne Google-Prüfung erscheint beim Verbinden der Hinweis „App nicht
überprüft“, den man für den eigenen Client bestätigen kann.

**Notion Calendar:** hat keine eigene Schnittstelle, zeigt aber verbundene
Google- (und iCloud-)Kalender an. Einen dieser Kalender als Zielkalender wählen;
die Aufgaben erscheinen dann automatisch in Notion Calendar, Änderungen dort
fließen über Google zurück.

Danach beschreibbare Zielkalender auswählen (Apple und Google gemischt, höchstens
zehn), Auswahl speichern und automatischen Sync einschalten.

**Automatisch:** nach Aufgabenänderungen, beim App-Start und jede Minute, solange
die App läuft. „Jetzt synchronisieren“ startet einen zusätzlichen Durchlauf.
Bei Apple-Kalendern erfolgt der anschließende Abgleich mit iCloud durch macOS
und kann zusätzliche Zeit brauchen; direkt verbundene Google-Kalender werden
sofort über die API geschrieben. Angezeigt werden letzter erfolgreicher Sync, Anzahl
synchronisierter Aufgaben und Fehler. Das Feature ist anfangs ausgeschaltet;
außerhalb von macOS steht nur Google zur Verfügung.

**Konflikte:** Ändern sich Aufgabe und Kalendertermin gleichzeitig, gewinnt der
neuere Stand. Bei mehreren Kalendern gewinnt die neueste geänderte Kalenderkopie;
bei gleichem Zeitstempel hat die Aufgabe Vorrang. Eine während des Lesens eintreffende
Aufgabenänderung wird nicht überschrieben. Während des Schreibens geänderte
Kalendertermine werden beim nächsten Durchlauf erneut geprüft.

**Entfernen:** Erledigte, gelöschte oder undatierte Aufgaben verlieren ihre
Kalenderkopien. Im Kalender gelöschte Termine löschen die Aufgabe nicht. Nach
zwei erfolgreichen Prüfungen mit mindestens einer Minute Abstand bleibt diese
Kalenderkopie unterdrückt, bis Titel, Notizen, Datum oder Uhrzeiten der Aufgabe geändert
werden. Ausschalten des Syncs oder Abwählen eines Kalenders lässt bestehende
Termine stehen; beim erneuten Aktivieren wird der Abgleich fortgesetzt.

Wiederholende Termine werden noch nicht unterstützt. Betroffene Aufgaben
werden mit konkretem Hinweis übersprungen; andere Aufgaben werden weiter
synchronisiert. Fehlende Kalender, entzogener Zugriff oder ein nicht erreichbares Google stoppen
den Durchlauf und werden nicht als Terminlöschungen behandelt.

## Sortieren und Filtern

Ein Symbol im Kopf der Liste öffnet ein Panel mit beidem. Ein Punkt daran zeigt,
dass für diese Ansicht etwas eingestellt ist.

**Sortieren nach** Reihenfolge, Fälligkeit, geplantem Datum, Priorität, Titel,
Erstellung oder Abschluss — auf- oder absteigend. „Reihenfolge" ist der
Standard und heißt: die Sortierung, die die Ansicht selbst vorgibt. Aufgaben
ohne Datum stehen immer am Ende, in beide Richtungen — eine Aufgabe ohne Termin
soll nicht über einer überfälligen stehen.

**Filtern** nach Projekten (auch „ohne Projekt"), Prioritäten (auch „ohne
Priorität"), Status und Fälligkeitsfenster. Nichts angekreuzt heißt: alles. Der
Statusfilter ersetzt die Vorgabe der Ansicht, statt sie zu ergänzen — nur so
lässt sich in der Inbox auch Erledigtes hervorholen.

**Pro Ansicht gemerkt.** „Heute" nach Fälligkeit und ein Projekt in eigener
Reihenfolge schließen sich nicht aus; jede Ansicht behält ihre Einstellung.

**Drag-and-drop.** Am Griff einer Aufgabe ziehen, um sie in der Liste zu
verschieben. Nur verschobene Aufgaben bekommen eine feste Position; die übrigen
folgen weiter der gewählten Sortierung. Die Positionen werden je Ansicht
gespeichert und bleiben nach einem Neustart erhalten. Beim Ziehen scrollt die
Liste am Rand automatisch. Mit `Esc` lässt sich die Bewegung abbrechen; auf
dem fokussierten Griff verschieben `↑` und `↓` die Aufgabe per Tastatur,
`Home` und `End` an den Anfang bzw. ans Ende.

**Schneller Reset.** Sobald eigene Positionen bestehen, steht direkt über der
Liste ein **Reset**-Knopf. Ein Klick entfernt alle festen Positionen dieser
Ansicht und stellt die gewählte Sortierung wieder her. Filter und Presets bleiben
erhalten. Im Menü einer verschobenen Aufgabe setzt **Position zurücksetzen**
nur diese eine Ausnahme zurück.

**Presets** sind benannte Paare aus Filter und Sortierung. Gewählt werden sie
im Panel, benannt und verwaltet in den Einstellungen. Weicht man nach der Wahl
von einem Preset ab, zeigt das Panel „Angepasst" und bietet an, daraus ein neues
zu sichern.

Filtert eine Einstellung alles weg, sagt der Leerzustand das ausdrücklich und
bietet einen Knopf zum Zurücksetzen — eine leere Liste soll nicht wie
Datenverlust aussehen.

## Projekte

- Frei anlegbar, mit **eigener Farbe** und einem **Symbol** aus der
  Lucide-Bibliothek.
- Optional mit Beschreibung und Fälligkeitsdatum.
- **Archivieren** statt löschen, wenn ein Projekt abgeschlossen ist.
- Die Navigation zeigt je Projekt die Zahl der offenen Aufgaben.

## Prioritäten

Die Stufen sind keine feste Vorgabe, sondern Teil der Einstellungen:
Bezeichnung, Farbe und Reihenfolge sind frei. Ausgeliefert wird mit „Niedrig",
„Mittel" und „Hoch".

Die Priorität einer Aufgabe zeigt sich als farbige Kante links an der Zeile.
Wird eine Stufe gelöscht, werden die Aufgaben, die auf sie zeigten, mit
aufgeräumt — die Kennungen der übrigen Stufen bleiben dabei stabil, es
verrutscht nichts.

## Erscheinungsbild

- **Hell, dunkel oder nach Systemeinstellung.** Bei „System" folgt die App dem
  Wechsel im laufenden Betrieb, auch im Detailfenster.
- **Eigene Flächenfarbe** für die Sidebar. Ist die gewählte Farbe dunkel,
  schaltet die Sidebar intern auf den dunklen Token-Satz um — Schrift, Rahmen
  und Nebentöne bleiben dadurch lesbar, statt nur der Hintergrund zu wechseln.
- **Deckkraft frei einstellbar.** Die Sidebar liegt mit Weichzeichner über dem,
  was darunter ist.
- **Eigene Farbe für abgehakte Kästchen.** Der Haken darin wird automatisch
  schwarz oder weiß — je nachdem, was auf der gewählten Farbe lesbar ist.

## Tastatur

Zwei globale Hotkeys, beide in den Einstellungen frei belegbar. Sie wirken aus
jeder App heraus:

| Standard | Wirkung |
|---|---|
| `⌘⇧T` | Sidebar auf- und zuklappen |
| `⌘⇧N` | Sidebar aufklappen und sofort eine Aufgabe anlegen |

> `⌘⇧N` überlagert im Finder „Neuer Ordner". Wer das braucht, stellt den Hotkey
> in den Einstellungen auf etwas anderes um.

Innerhalb der Fenster:

| Taste | Wirkung |
|---|---|
| `⌘F` | In der Sidebar: suchen. Im Detailfenster: Fenster vergrößern |
| `↵` | In der Schnellzeile: Aufgabe anlegen |
| `⌘↵` | In der Schnellzeile: ausführlich anlegen. Im Detailfenster: speichern |
| `Esc` | Schnellzeile verwerfen, Suche oder Detailfenster schließen |

Fokusringe bleiben überall sichtbar — die Sidebar ist vollständig mit der
Tastatur bedienbar.

## Benachrichtigungen

Unter **Einstellungen → Benachrichtigungen** lassen sich **native Popups** und
**E-Mails** unabhängig voneinander einschalten. Beide sind zunächst aus.

- **Erinnerungszeit.** Offene Aufgaben werden am Fälligkeitstag zur gewählten
  lokalen Uhrzeit gemeldet. Aufgaben ohne Fälligkeitsdatum lösen nichts aus.
  Offene überfällige Aufgaben werden beim nächsten Start nachgeholt.
- **Einmal je Aufgabe, Termin und Kanal.** Die Zustellhistorie liegt in SQLite
  und bleibt nach einem Neustart erhalten. Ein neuer Fälligkeitstermin kann
  erneut erinnern. Mehrere Aufgaben werden zu einer Mitteilung zusammengefasst.
- **Native Popups.** Mitteilungen von macOS oder Windows. Anzeige und Ton
  hängen zusätzlich von den Systemeinstellungen ab. Ein Testknopf übergibt
  eine Probemitteilung; Windows-Popups sollten in der installierten App geprüft
  werden.
- **E-Mail einrichten.** SMTP-Host, Port, STARTTLS oder TLS, Benutzername,
  Absender- und Empfängeradresse. Das Passwort bzw. App-Passwort liegt im
  System-Schlüsselbund, niemals in der Datenbank. Zugang speichern, Test-E-Mail
  senden und den E-Mail-Kanal einschalten. Der Zugang lässt sich samt Passwort
  wieder entfernen.
- **Fehler sichtbar.** Fehlgeschlagene E-Mail-Zustellungen erscheinen in diesem
  Einstellungsbereich und werden nach fünf Minuten erneut versucht. Popups und
  E-Mails behalten jeweils ihre eigene Zustellhistorie.

Erinnerungen werden alle 30 Sekunden geprüft, solange **die App läuft** — auch
bei eingeklappter Sidebar. Erledigte und gelöschte Aufgaben, Unteraufgaben
erledigter Hauptaufgaben und Aufgaben in archivierten Projekten werden ausgelassen.

## Unter der Haube

**Lokale Daten.** Eine SQLite-Datei, kein App-Konto, kein eigener Server. Die
Content-Security-Policy erlaubt im Frontend keine externen Verbindungen; auch
die Schriftart liegt als lokale Datei bei. Nur aktivierte E-Mail-Erinnerungen
und ausdrücklich ausgelöste Test-E-Mails verbinden das Rust-Backend mit dem
eingerichteten SMTP-Server und übertragen Aufgabentitel und Fälligkeitsdaten.

**Das Detailfenster bleibt am Leben.** Beim Schließen wird es nur versteckt statt
zerstört, und es wird erst eingeblendet, wenn Daten und Theme vollständig stehen.
Deshalb erscheint es beim Öffnen fertig, statt sichtbar nachzuladen.

**Nur `transform` und `opacity` werden animiert.** Breite oder Position zu
animieren würde bei jedem Bild das Layout neu berechnen; das Auf- und Zuklappen
nutzt darum bewusst `translateX`.

**Eine Instanz.** Ein zweiter Start bringt die bestehende Sidebar nach vorn,
statt eine weitere zu öffnen.
