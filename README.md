# todoSideBar

> A minimalistic Sidebar for Desktop-Devices to manage task comfortable and easy
> without overwhelm and full customisation possiblities.

Eine Aufgabenliste, die am rechten Bildschirmrand wohnt statt in einem eigenen
Fenster. Ein schmaler Griff bleibt immer sichtbar; ein Klick oder ein globaler
Hotkey klappt die Spalte auf, ein weiterer wieder zu. Für alles, was in einer
220 px breiten Spalte keinen Platz hat, öffnet sich ein zweites Fenster.

Gebaut mit [Tauri 2](https://tauri.app), React 19 und SQLite. Die Daten liegen
ausschließlich lokal — kein Konto, kein Server, keine Netzwerkverbindung.

**→ [Was die App kann, im Einzelnen](FEATURES.md)**

---

## Voraussetzungen

| | Version | Wofür |
|---|---|---|
| [Node.js](https://nodejs.org) | 20 oder neuer | Frontend-Build |
| [pnpm](https://pnpm.io) | 10 oder neuer | Paketverwaltung (Workspace) |
| [Rust](https://rustup.rs) | 1.85 oder neuer | Edition 2024 wird vorausgesetzt |

`pnpm` lässt sich nach der Node-Installation mit `corepack enable` aktivieren,
ohne es separat zu installieren.

### Zusätzlich unter macOS

Ab macOS 11 (Big Sur). Die Xcode-Kommandozeilenwerkzeuge liefern den Linker:

```bash
xcode-select --install
```

### Zusätzlich unter Windows

Windows 10 (1803) oder 11. Zwei Dinge fehlen dort meist:

- **Microsoft C++ Build Tools** — im
  [Visual Studio Installer](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  die Arbeitslast „Desktopentwicklung mit C++" auswählen.
- **WebView2 Runtime** — auf Windows 11 vorinstalliert, für Windows 10
  [hier](https://developer.microsoft.com/microsoft-edge/webview2/) als
  „Evergreen Standalone Installer".

## Installation

```bash
git clone https://github.com/P4YM3NT/todoSideBar_v2.git
cd todoSideBar_v2
pnpm install
```

Der erste Build übersetzt die gesamte Rust-Abhängigkeitskette und dauert je nach
Maschine einige Minuten. Jeder weitere ist deutlich schneller.

```bash
pnpm build
```

Das fertige Paket landet unter:

| Plattform | Pfad |
|---|---|
| macOS | `target/release/bundle/macos/todoSideBar.app` und `…/dmg/` |
| Windows | `target/release/bundle/msi/` und `…/nsis/` |

Unter macOS die `.app` nach `/Programme` ziehen, unter Windows den Installer
ausführen.

### Hinweis für macOS: unsignierte Builds

Ein selbst gebautes Paket ist nicht signiert und nicht notarisiert. macOS
verweigert den ersten Start deshalb mit „… kann nicht geöffnet werden".
Entweder einmalig über **Systemeinstellungen → Datenschutz & Sicherheit →
Trotzdem öffnen** freigeben, oder das Quarantäne-Merkmal direkt entfernen:

```bash
xattr -dr com.apple.quarantine /Applications/todoSideBar.app
```

## Entwicklung

```bash
pnpm dev         # Tauri im Entwicklungsmodus, mit Hot Reload im Frontend
pnpm typecheck   # TypeScript prüfen
pnpm test        # Rust-Tests des Workspace
```

## Wo die Daten liegen

Eine einzelne SQLite-Datei, `todo.db`:

| Plattform | Pfad |
|---|---|
| macOS | `~/Library/Application Support/de.ptr-digital.todosidebar/` |
| Windows | `%APPDATA%\de.ptr-digital.todosidebar\` |

Für ein Backup genügt es, diese Datei zu kopieren. Zum vollständigen
Zurücksetzen der App reicht es, sie zu löschen — sie wird beim nächsten Start
neu angelegt.

## Aufbau des Projekts

```
crates/todo-core/        Datenmodell, SQLite-Zugriff, Einstellungen — ohne UI
apps/sidebar/src/        React-Frontend (Sidebar und Detailfenster teilen ein Bundle)
apps/sidebar/src-tauri/  Fenstergeometrie, globale Hotkeys, Tray, IPC
```

`todo-core` kennt weder Tauri noch React. Die Trennung ist Absicht: eine spätere
vollwertige App soll dieselben Bausteine benutzen können.

Die verbindlichen UI-Regeln — Komponentenpflicht, Abstände, Typografie, Farben —
stehen in [`.claude/skills/design-system/SKILL.md`](.claude/skills/design-system/SKILL.md).

## Stand

Version 0.1.0. Entwickelt und getestet unter macOS. Der Windows-Pfad ist
implementiert (eigene Fensterstile statt des dort fehlenden NSPanel-Äquivalents),
aber bislang nicht auf echter Hardware erprobt — Rückmeldungen dazu sind
willkommen.

## Lizenz

MIT
