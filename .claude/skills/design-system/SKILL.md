---
name: design-system
description: Verbindliche UI-Regeln für todoSideBar - Komponentenpflicht, Abstände, Typografie, Farben, Dichte. Lies das VOR jeder Änderung an UI-Code (JSX/TSX/CSS) und bevor du eine Komponente neu baust.
---

# Design-System todoSideBar

Zwei Oberflächen, unterschiedliche Dichte:

| | Sidebar | Detailfenster |
|---|---|---|
| Breite | ~220-400 px, Standard 15 % | ~600×500 px |
| Dichte | kompakt | großzügig |
| Basis-Textgröße | `text-sm` (14 px) | `text-sm`, Überschrift `text-base` |

Alles hier ist verbindlich. Abweichungen brauchen einen Kommentar im Code, der die Abweichung begründet.

---

## 1. Komponentenpflicht

**Baue nie selbst, was es als Komponente gibt.** Vor jedem interaktiven Element oder erkennbaren UI-Muster:

1. Schau in `apps/sidebar/src/components/ui/`.
2. Nicht da? Hinzufügen: `pnpm dlx shadcn@latest add <name>`
3. Erst wenn nachweislich nichts passt, selbst bauen - **mit Kommentar, warum**.

Das gilt für **alle** Komponenten, ausnahmslos: Buttons, Eingabefelder, Checkboxen, Dialoge, Menüs, Tooltips, Datumsauswahl, Umschalter, Tabs, Trennlinien, Scrollbereiche, Badges.

Was wir brauchen und wie es bei shadcn heißt:

| Zweck | `shadcn add …` |
|---|---|
| Aufgabe abhaken | `checkbox` |
| Titel/Felder eingeben | `input`, `textarea` |
| Projekt wählen | `select` |
| Termin wählen | `calendar`, `popover` |
| Kontextmenü an Aufgabe | `dropdown-menu` |
| Detailfenster-Aufbau | `dialog` (bzw. Fensterinhalt), `separator`, `scroll-area` |
| Griff-/Icon-Erklärung | `tooltip` |
| Einstellungen an/aus | `switch` |
| Priorität, Tags | `badge` |
| Löschen bestätigen | `alert-dialog` |
| Fehler beim Speichern | `sonner` |

**Icons ausschließlich aus `lucide-react`.** Keine handgeschriebenen SVG-Pfade im JSX - sie sind inkonsistent in Strichstärke und Optik und lassen sich nicht zentral ändern.

---

## 2. Abstände

Erlaubt sind **nur** diese Stufen. Keine anderen, und niemals beliebige Werte wie `p-[13px]`.

| Stufe | px | Wofür |
|---|---|---|
| `1` | 4 | Icon zu Text, innerhalb eines Labels |
| `2` | 8 | innerhalb einer Zeile oder Steuergruppe |
| `3` | 12 | zwischen Listeneinträgen, Sidebar-Innenabstand |
| `4` | 16 | zwischen Abschnitten in der Sidebar |
| `6` | 24 | zwischen Abschnitten im Detailfenster, dessen Innenabstand |
| `8` | 32 | nur als großer Block-Trenner im Detailfenster |

Verboten: `5`, `7`, `9`, `10` und alles darüber. Wenn eine dieser Stufen nötig scheint, stimmt die Hierarchie nicht.

Feste Zuordnungen:

```
Sidebar-Container      p-3
Listeneintrag          px-3 py-2
Abstand Listeneinträge space-y-2
Inhalt im Eintrag      gap-3 (Checkbox zu Text), space-y-1 (Zeilen)
Sidebar-Abschnitte     space-y-4
Detailfenster          p-6
Detail-Feldgruppen     space-y-6
Label zu Feld          space-y-2
```

---

## 3. Typografie

Größe wird nach **Rolle** gewählt, nie nach Optik.

| Klasse | px | Rolle |
|---|---|---|
| `text-xs` | 12 | Metadaten: Datum, Anzahl, Hinweise, Leerzustände |
| `text-sm` | 14 | **Standard.** Aufgabentitel, Eingaben, Buttons, Menüs |
| `text-base` | 16 | Abschnittsüberschriften im Detailfenster |
| `text-lg` | 18 | **nur** der Aufgabentitel im Detailfenster |

`text-xl` und größer kommen nicht vor. In einer 220 px breiten Spalte wirken sie sofort unproportioniert. `text-lg` ist allein dem Aufgabentitel im Detailfenster vorbehalten - dort ist Platz und er trägt die Hierarchie.

Schriftschnitte:

- `font-normal` - Fließtext, Aufgabentitel
- `font-medium` - Labels, Buttons, aktive Navigation, Überschriften
- `font-semibold` - **nur** die Überschrift im Detailfenster
- `font-bold` - nie

Zeilenhöhe: `leading-snug` bei mehrzeiligen Titeln, sonst Standard.

Umbruch: Aufgabentitel in der Liste **einzeilig mit `truncate`**, Beschreibung `line-clamp-1`. Eine zweite Titelzeile lässt die Zeilenhöhe springen und macht die Liste unruhig - die vollständige Fassung steht im Detailfenster.

Schrift: Geist (kommt als lokale `woff2` mit shadcn). Keine Web-Fonts von externen Hosts - die CSP blockt sie.

---

## 4. Farben

**Nur Design-Tokens. Nie rohe Tailwind-Paletten** wie `text-neutral-400` oder `bg-neutral-900` - die brechen Hell/Dunkel und lassen sich nicht zentral ändern.

| Token | Wofür |
|---|---|
| `bg-background` / `text-foreground` | Grundfläche und Text |
| `bg-card` | Sidebar- und Fensterfläche |
| `text-muted-foreground` | Metadaten, Platzhalter, Erledigtes |
| `bg-accent` / `text-accent-foreground` | Hover, aktive Zeile |
| `border-border` | alle Trennlinien und Rahmen |
| `bg-primary` / `text-primary-foreground` | primäre Aktion |
| `text-destructive` | Löschen, Fehler, Überfälliges |
| `ring-ring` | Fokus |

**Ausnahme:** Der Fensterhintergrund muss transparent bleiben - siehe `src/styles.css`. `bg-background` darf nicht auf `body` liegen, sonst ist der Overlay-Effekt weg.

Projektfarben sind Nutzerdaten und kommen als Inline-Style aus der Datenbank, nicht als Tailwind-Klasse.

---

## 5. Dichte, Radius, Bewegung

**Dichte:** In der Sidebar immer die kompakte Variante - `size="sm"` bei Buttons und Eingaben. shadcn-Standardgrößen sind für breite Web-Layouts gedacht und sprengen die Spalte.

**Radius:** `rounded-md` für Zeilen, Felder und Buttons. `rounded-lg` für Karten und das Detailfenster. Nichts anderes.

**Bewegung:** 150-200 ms, nur `transform`, `opacity` und Farben. Niemals `width`, `height` oder `left`/`top` animieren - das ruckelt und kostet Layout-Durchläufe. Das Auf- und Zuklappen der Sidebar nutzt bewusst `translateX`.

**Fokus:** Fokusringe nie entfernen. Die Sidebar wird per Tastatur bedient.

---

## 6. Dokumentierte Ausnahmen

Stand: alle Altlasten sind migriert. Es gibt genau eine bewusste Abweichung:

- `src/components/Handle.tsx` nutzt ein rohes `<button>` statt der `button`-Komponente.
  Der Griff ist 14x100 px - keine Groessenvariante trifft das, und alle bringen
  Innenabstaende mit, die hier nicht passen. Die Begruendung steht als Kommentar
  in der Datei.

- `src/components/ThemeToggle.tsx` nutzt ein handgeschriebenes SVG statt Lucide.
  Sonne und Mond sollen sich ineinander verwandeln; mit zwei fertigen Icons ginge
  nur ein Ueberblenden. Animiert werden weiterhin nur `transform` und `opacity`.

- `src/components/ui/checkbox.tsx` weicht an zwei Stellen vom shadcn-Original ab.
  Grund ist beide Male die Spezifitaet: `dark:` erzeugt `:is(.dark *)` und zaehlt
  als zweite Klasse, `data-checked:` erzeugt `:where(...)` und zaehlt null. Jede
  `dark:`-Regel schlaegt damit jede `data-checked:`-Regel - unabhaengig von der
  Reihenfolge und von dem, was `cn()` am Aufrufort setzt.
  `dark:bg-input/30` ist deshalb auf `dark:not-data-checked:bg-input/30`
  eingeschraenkt, und `dark:data-checked:bg-primary` ist entfernt. Sonst ist die
  abgehakte Box in Dunkel entweder weiss oder gar nicht gefuellt, statt in der
  eingestellten Akzentfarbe. Nach `shadcn add checkbox` wieder herstellen.

  **Regel daraus:** eine `dark:`-Variante nie fuer etwas verwenden, das eine
  Zustandsklasse (`data-checked:`, `hover:`, …) spaeter uebersteuern soll. Die
  Tokens sind bereits themenabhaengig - `dark:` ist dafuer ohnehin unnoetig.

Neue Abweichungen gehoeren hierher **und** als Kommentar an die Stelle im Code.

## 7. Pruefen

Vor dem Abschluss einer UI-Aenderung, aus `apps/sidebar/`:

```bash
grep -rnE "(neutral|blue|red|gray|slate|zinc)-[0-9]{3}" src --include='*.tsx' | grep -v "src/components/ui/"
grep -rn "<svg" src --include='*.tsx' | grep -v "src/components/ui/"
grep -rnE "\b(p|px|py|m|mx|my|gap|space-[xy])-(5|7|9|1[0-9])\b" src --include='*.tsx' | grep -v "src/components/ui/"
grep -rnE "text-(lg|xl|2xl|3xl)" src --include='*.tsx' | grep -v "src/components/ui/"
```

Jede Ausgabe ist ein Verstoss. `src/components/ui/` ist ausgenommen - das ist
generierter shadcn-Code.
