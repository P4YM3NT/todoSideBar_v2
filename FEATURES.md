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

## Unter der Haube

**Alles lokal.** Eine SQLite-Datei, kein Konto, kein Server. Die
Content-Security-Policy erlaubt keine externen Verbindungen; auch die Schriftart
liegt als lokale Datei bei.

**Das Detailfenster bleibt am Leben.** Beim Schließen wird es nur versteckt statt
zerstört, und es wird erst eingeblendet, wenn Daten und Theme vollständig stehen.
Deshalb erscheint es beim Öffnen fertig, statt sichtbar nachzuladen.

**Nur `transform` und `opacity` werden animiert.** Breite oder Position zu
animieren würde bei jedem Bild das Layout neu berechnen; das Auf- und Zuklappen
nutzt darum bewusst `translateX`.

**Eine Instanz.** Ein zweiter Start bringt die bestehende Sidebar nach vorn,
statt eine weitere zu öffnen.
