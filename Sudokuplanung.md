# clisudoku — Spec

## Tech-Stack
- **Sprache:** Rust
- **Terminal-Library:** crossterm
- **Konfigformat:** TOML + serde
- **Datenspeicherung:** SQLite
- **Netzwerk:** TCP, tokio als async Runtime
- **Plattformen:** Windows, Linux, Mac gleichwertig

---

## Grafik & Darstellung
- **Stil:** Hell auf Dunkel, minimalistisch, platzsparend
- **Notizmodus:** Kandidaten als `1 2 3 / 4 5 6 / 7 8 9` direkt in der Zelle
- **Gitter:** Drei Liniengewichte — Außenrahmen (doppelt), Box-Grenzen (schwer), Zell-Trenner (dünn); siehe Grid-Design unten

### Ziffern-Stile
Zwei eingebaute Stile, in Config wählbar:
- **`retro`** — Halbblock-Unicode (`▞▀▚` etc.), organisch-geschwungen; Designs festgelegt (siehe unten)
- **`awkward-retro`** — Vollblock (`█`), kantig-pixelig; Designs festgelegt (siehe unten)

#### `retro` — Ziffern-Designs (3×3 Zeichen, Halbblock-Unicode)

```
1      2      3      4      5      6      7      8      9
▗▐     ▞▀▚    ▞▀▚    ▌ ▐    ▛▀▀    ▞▀     ▀▀▞    ▞▀▚    ▞▀▚
 ▐      ▞       ▚    ▀▀▜    ▀▀▚    ▛▀▚     ▞     ▚▄▞    ▚▄▞
 ▐     ▟▄▄    ▚▄▞      ▐    ▚▄▞    ▚▄▞    ▞      ▚▄▞     ▞
```

Stile sind **erweiterbar und austauschbar**:
- Jeder Stil definiert die Darstellung aller Ziffern 1–9 in allen Zuständen (normal, aktiv, vorgegeben, Fehler, gelöst)
- Custom-Stile können im Config-Verzeichnis abgelegt werden und werden automatisch erkannt
#### `awkward-retro` — Ziffern-Designs (3×3 Zeichen, Vollblock)

```
1       2      3      4      5    6   7      8      9
 ██     ██     ██    █ █    ██   █    ███    ██    ███
  █      █      ██   ███    █    ██     █   ███    ███
 ███     ██    ██      █   ██    ██     █   ███      █
```

- In Config: `digit_style = "retro"` / `"awkward-retro"` / `"mein-stil"`

*Hinweis: Das Grid-Design-Beispiel zeigt die Gitterstruktur mit `retro`-Ziffern.*

### Grid-Design

Drei Liniengewichte:
- **Außenrahmen:** Doppellinie (`╔═║╚╝╗`, `╤`/`╧` oben/unten für alle Spalten)
- **Box-Trennlinien:** Schwere Linie (`┃` vertikal, `━` horizontal, `╋`/`┿` Kreuzungen)
- **Zell-Trennlinien innerhalb Box:** Dünne Linie (`│` vertikal, `─` horizontal, `┼`/`╂` Kreuzungen)

Jede Zelle: 7 Zeichen breit, 3 Zeilen hoch — Notiz-Kandidaten (`1 2 3 / 4 5 6 / 7 8 9`) oder Ziffern-Grafik.

**Voraussetzung:** Monospaced-Schrift im Terminal — bei Proportionalschrift bricht das Grid. Hinweis in Doku und Help-Screen.

```
╔═══════╤═══════╤═══════╤═══════╤═══════╤═══════╤═══════╤═══════╤═══════╗
║ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ║
║ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ║
║ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ║
╟───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────╢
║ 1 2 3 │  ▞▀▚  │ 1 2 3 ┃ 1 2 3 │  ▗▐   │ 1 2 3 ┃ 1 2 3 │  ▌ ▐  │ 1 2 3 ║
║ 4 5 6 │    ▚  │ 4 5 6 ┃ 4 5 6 │   ▐   │ 4 5 6 ┃ 4 5 6 │  ▀▀▜  │ 4 5 6 ║
║ 7 8 9 │  ▚▄▞  │ 7 8 9 ┃ 7 8 9 │   ▐   │ 7 8 9 ┃ 7 8 9 │    ▐  │ 7 8 9 ║
╟───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────╢
║ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ║
║ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ║
║ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ║
╟━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━╢
║ 1 2 3 │ 1 2 3 │  ▞▀▚  ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ║
║ 4 5 6 │ 4 5 6 │   ▞   ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ║
║ 7 8 9 │ 7 8 9 │  ▟▄▄  ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ║
╟───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────╢
║ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │  ▞▀▚  ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ║
║ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │  ▚▄▞  ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ║
║ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │  ▚▄▞  ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ║
╟───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────╢
║ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │  ▞▀▚  ║
║ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │  ▚▄▞  ║
║ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │   ▞   ║
╟━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━╋━━━━━━━┿━━━━━━━┿━━━━━━━╢
║ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │  ▛▀▀  │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ║
║ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │  ▀▀▚  │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ║
║ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │  ▚▄▞  │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ║
╟───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────╢
║ 1 2 3 │  ▞▀   │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ║
║ 4 5 6 │  ▛▀▚  │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ║
║ 7 8 9 │  ▚▄▞  │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ║
╟───────┼───────┼───────╂───────┼───────┼───────╂───────┼───────┼───────╢
║ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │ 1 2 3 │ 1 2 3 ┃ 1 2 3 │  ▀▀▞  │ 1 2 3 ║
║ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │ 4 5 6 │ 4 5 6 ┃ 4 5 6 │   ▞   │ 4 5 6 ║
║ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │ 7 8 9 │ 7 8 9 ┃ 7 8 9 │  ▞    │ 7 8 9 ║
╚═══════╧═══════╧═══════╧═══════╧═══════╧═══════╧═══════╧═══════╧═══════╝
```

### Farbsystem

Jeder Zustand hat eine **eigene, frei konfigurierbare Farbe** (Vordergrund + Hintergrund). Zuweisung über Settings-Screen in logischen Gruppen (je ein Frame). Sinnvolle Defaults vorbelegt.

#### Frame 1 — Hintergrund & Gitter
| Schlüssel | Bedeutung |
|-----------|-----------|
| `ui.background` | Allgemeiner Hintergrund |
| `grid.border` | Außenrahmen (doppelt) |
| `grid.box` | Box-Trennlinien (schwer) |
| `grid.cell` | Zell-Trennlinien innerhalb Box (dünn) |

#### Frame 2 — Zellen
| Schlüssel | Bedeutung |
|-----------|-----------|
| `cell.normal` | Leere oder befüllte Zelle, Standard |
| `cell.active` | Aktuell ausgewählte Zelle |
| `cell.active_box` | Box der aktiven Zelle |
| `cell.active_cross` | Reihe + Spalte der aktiven Zelle (per Toggle de/aktivierbar) |
| `cell.hint_target` | Vom Hint-System markierte Zielzelle |
| `cell.hint_source` | Vom Hint-System markierte Quellzelle(n) |

#### Frame 3 — Ziffern
| Schlüssel | Bedeutung | Default-Intensität |
|-----------|-----------|-------------------|
| `digit.given` | Vorgegebene Ziffer (unveränderlich) | sehr hell |
| `digit.user` | Vom User eingetragene Ziffer (korrekt / nicht validiert) | hell |
| `digit.error` | Eingetragene Ziffer mit Konflikt (nur wenn Fehleranzeige aktiv) | rot |
| `digit.highlight` | Gleiche Ziffer wie in aktiver Zelle | hell + Hervorhebungsfarbe |
| `digit.hint_target` | Ziffer in Hint-Zielzelle | Akzentfarbe |

Unterscheidung erfolgt primär über **Farbintensität** — gleiche Zeichen, kein separater Zeichensatz nötig.

#### Frame 4 — Notizen
| Schlüssel | Bedeutung |
|-----------|-----------|
| `note.normal` | Kandidat normal |
| `note.highlight` | Gleiche Kandidaten-Ziffer wie aktive Zelle |
| `note.eliminated` | Ungültiger Kandidat (nur wenn Auto-Update deaktiviert) |

#### Frame 5 — Benutzeroberfläche
| Schlüssel | Bedeutung |
|-----------|-----------|
| `ui.text` | Normaler Text (Status, Labels) |
| `ui.text_dim` | Gedimmter Text (inaktive Elemente) |
| `ui.text_highlight` | Hervorgehobener Text |
| `ui.cursor` | Cursor in Menüs und Settings |
| `ui.button` | Buttons im Maus-Panel |
| `ui.button_active` | Aktiver/gedrückter Button |
| `ui.button_disabled` | Deaktivierter Button (z.B. Hint im Multiplayer) |

---

## Screens
Vier Screens, über Tasten erreichbar:

**Startbildschirm:**
Erscheint beim Programmstart und nach `Escape` aus dem Spiel.
- `[Weiterspielen]` — nur sichtbar wenn Spielstand vorhanden
- `[Neues Spiel]` → Schwierigkeitsauswahl (`Einfach` / `Mittel` / `Schwer`), dann Start
- `[Settings]`
- `[Beenden]`
- Navigation mit `↑↓`, Auswahl mit `Enter`

---

**Game-Screen:**
- Grid und Status-Panel nebeneinander
- Umschalten zwischen Numpad- und Maus-Modus: `m`

Status-Panel je nach Kombination:

| Modus | Inhalt |
|-------|--------|
| Solo + Numpad | Statistiken (Streak, Lösungsquote) + abgelaufene Zeit |
| Solo + Maus | Ziffernauswahlpad + Sonderfunktionen (Undo/Redo/Löschen/Hint/Help/Settings) + abgelaufene Zeit |
| Multiplayer + Numpad | Spielerübersicht mit Fortschritt aller Teilnehmer |
| Multiplayer + Maus | Ziffernauswahlpad + Sonderfunktionen (Undo/Redo/Löschen/Help/Settings, kein Hint) + nur der führende Spieler mit Fortschritt |

**Settings-Screen (`s`):**
- Alle Config-Optionen direkt im Programm bearbeitbar
- Optionen in logischen **Frames** gegliedert; `←→` zwischen Haupt-Frames, `↑↓` innerhalb eines Frames
- Farb-Unter-Frames direkt mit Pfeiltasten navigierbar (kein separater Auswahlschritt)
- `←→` auch für Enum/Toggle-Optionen (sofortige Vorschau)
- `Enter` für Texteingabe-Editiermodus (z.B. Farb-Hex-Werte)
- Änderungen beim Verlassen mit Bestätigung übernehmen (i18n: Ja/Nein, Yes/No etc.)
- Keine Änderungen → direkt zurück ohne Bestätigung
- `s` oder `Escape` → zurück

Settings-Frames:

| Frame | Inhalt |
|-------|--------|
| **Darstellung** | Ziffern-Stil (`retro` / `awkward-retro` / custom), Sprache |
| **Farben** | 5 Unter-Frames: Hintergrund & Gitter / Zellen / Ziffern / Notizen / UI |
| **Steuerung** | Alle Keybindings via Press-to-bind |
| **Spielverhalten** | Passive Hilfen (3 Toggles), Aktive Hilfen (6 Toggles), Fehleranzeige (`flash`/`beep`/`silent`), `cell.active_cross` Toggle, Challenge-Modus (Toggle + max. Fehleranzahl) |
| **Netzwerk** | Port, Standard-Spielername (für Multiplayer) |

**Help-Screen (`h`):**
- Tastenbelegungen dynamisch aus aktueller Config generiert
- Spielmodi und Strategien erklärt
- Config-Dateipfad angezeigt
- `h` oder `Escape` → zurück

---

## Steuerung

### Tastatur — Numpad (einhändig)

**Navigation (2-stufig):**
- `Enter` → Nav-Modus aktiv
- `Numpad 1–9` → Box wählen (Layout spiegelt Sudoku-Gitter, `5` = Mitte)
- `Numpad 1–9` → Zelle innerhalb der Box wählen → Eingabe-Modus aktiv
- `Enter` mid-Navigation → Reset, neu von Box-Auswahl
- `Enter` aus Eingabe-Modus → zurück zu Nav

**Pfeiltasten** → zellweise Navigation, jederzeit nutzbar; Zelle wird direkt aktiv (Eingabe-Modus, kein `Enter` nötig)
- Überlauf: Zeilen/Spalten-Wrap (rechts von Spalte 9 → Spalte 1, gleiche Zeile)

**Eingabe-Modus** (bleibt aktiv bis nächstes `Enter`):
- `1–9` → Ziffer eintragen: Lösungs-Modus setzt/überschreibt, Notiz-Modus togglet Kandidat
- `0` → Lösungs/Notiz-Modus wechseln (jederzeit, auch während Eingabe)
- `-` → Zelle leeren (Lösung + alle Notizen) — Rückfrage vor Ausführung

**Erweitertes Menü:**
- `*` → Untermenü mit Funktionshinweisen und konfigurierbaren Bindings

| Taste | Funktion |
|-------|----------|
| `1` | Undo |
| `2` | Redo |
| `3` | Hook — reserviert für spätere Belegung |
| `4` | Hint |
| `5` | Hook — reserviert für spätere Belegung |
| `6` | Hook — reserviert für spätere Belegung |
| `7` | Hook — reserviert für spätere Belegung |
| `8` | Hook — reserviert für spätere Belegung |
| `9` | Hook — reserviert für spätere Belegung |

**Spiel-Steuerung:**
- `Escape` → Spiel beenden (Auto-Save), zurück zum Startbildschirm
- `Space` → Pause/Weiter-Toggle (Timer stoppt, Grid wird verdeckt, Overlay "Paused"; nochmals `Space` → weiter)

**Neues Spiel** (nur vom Startbildschirm aus):
- Schwierigkeitsgrad wählen → Spiel startet

**Modus-Umschaltung:**
- `m` → wechselt zwischen Numpad-Modus und Maus-Modus (Status-Panel passt sich an)

**Keybinding-Konfiguration:**
- Alle Tastenbelegungen konfigurierbar via Press-to-bind im Settings-Screen
- Liste aller Aktionen, Aktion auswählen → Taste drücken → gespeichert
- Löst Numpad-Orientierungsvarianten und Numpad-lose Tastaturen

---

### Maussteuerung

**Grid:** Zelle anklicken → auswählen

**Panel rechts:**
- Toggle-Button mit Zustandsanzeige: `[Lösung | Notiz]`
- Ziffern `1–9`: anklicken = eintragen je nach aktivem Modus; vollständig gesetzte Ziffern ausgegraut
- Buttons: `[Undo]` `[Redo]` `[Löschen]` — Löschen mit Rückfrage
- Buttons: `[Hint]` (nur Solo) `[Help]` `[Settings]`

---

## Konfiguration
Alles konfigurierbar, sinnvolle Defaults:
- Farben (Vordergrund + Hintergrund pro Zustand; alle Schlüssel siehe Farbsystem)
- Keybindings
- Ziffern-Stil (`retro` / `awkward-retro` / custom)
- Spielverhalten (Fehleranzeige, Hilfen etc.)
- Netzwerk (Port)
- Scoring-Modus
- Sprache

**Config-Datei Speicherort:** Plattform-Standard
- Linux: `~/.config/sudoku/config.toml`
- Windows: `AppData`
- Mac: `~/Library/Application Support`

---

## CLI-Args
- `-s "102300..."` → Rätsel als String eingeben (`0` oder `.` für leere Felder)
- `-f puzzle.txt` → aus Datei laden
- `--lang de` → Sprache setzen (überschreibt Config einmalig)
- `--host [--port 4242]` → Multiplayer-Host
- `--connect IP:PORT --name "Name"` → Multiplayer-Client
- `-h` → ausführliche Hilfe ausgeben, dann beenden

---

## Spiellogik

### Puzzle-Generierung:
- Backtracking + schrittweises Entfernen + Eindeutigkeitsprüfung
- Schwierigkeit wird aktiv gesteuert (Ansatz 2 — vorwärts beim Generieren)
- Bekannte Lösung als Validierung

### Schwierigkeitsskala:
| Schwierigkeit | Benötigte Strategien |
|---------------|---------------------|
| Einfach | Naked Single, Hidden Single |
| Mittel | + Naked Pair, Pointing Pair/Triple |
| Schwer | + Naked Triple/Quad, Hidden Pair/Triple, Box-Line Reduction, X-Wing |
| Experte | Phase 3 — Zukunft (in Spec dokumentiert) |

---

## Solver-Modul (Herzstück)
Eigenständiges Modul, genutzt von Generator, Hint-System und Validator:

```
solver/
  mod.rs
  candidates.rs
  naked_single.rs
  hidden_single.rs
  naked_pair.rs
  pointing_pair.rs
  naked_triple.rs
  hidden_pair.rs
  box_line_reduction.rs
  x_wing.rs
  backtracking.rs    ← stiller Fallback
```

---

## Support-System (nur Solo)
**Passive Hilfen** (jede einzeln ein/ausschaltbar):
- Gleiche Ziffern hervorheben
- Ungültige Eingaben markieren
- Notizen automatisch aktualisieren (beim Setzen einer Ziffer: Kandidat in Reihe/Spalte/Box entfernen)

**Aktive Hilfen (aufsteigend):**
1. Kandidaten füllen
2. Naked Single zeigen
3. Hidden Single zeigen
4. Nächsten logischen Schritt markieren (ohne Lösung zu verraten)
5. Einzelnes Feld lösen
6. Komplettlösung

- Jede Stufe in Config de/aktivierbar
- Nutzung wird in der **Statistik** vermerkt (Art und Häufigkeit)

---

## Falscheingaben
- Einstellbar: `flash` / `beep` / `silent`
- Beep-Einschränkungen in Doku vermerkt
- Bei vollständigem aber falschem Rätsel: Fehlermeldung + Option nur falsche Felder zu löschen

---

## Challenge-Modus

- Per Config aktivierbar (Toggle im Spielverhalten-Frame)
- Zählt jeden Fehler (falsch eingetragene Ziffer)
- Konfigurierbare maximale Fehleranzahl (z.B. 3); bei 0 = nur zählen, kein Abbruch
- Bei Erreichen des Limits: Spiel wird abgebrochen, Meldung, als **Fail** in DB gespeichert
- Perfektes Spiel (0 Fehler, gelöst) wird separat in DB markiert
- Im Multiplayer: jeder Spieler hat eigenen Fehlerzähler; Abbruch nur für den jeweiligen Spieler

---

## Auto-Save
- Beim Beenden automatisch speichern
- Beim Neustart: "Weiterspielen?"
- Bei Start mit `-n`, `-s`, `-f` oder `-d` und vorhandenem Spielstand: Rückfrage ob Spielstand verworfen werden soll

---

## Rätsel-Datenbank (SQLite)
Pro Rätsel:
- Original-Puzzle (81 Ziffern, unveränderlich)
- Spielstand (81 Ziffern + Notizen pro Zelle + abgelaufene Zeit) — für Auto-Save / Weiterspielen
- Schwierigkeitsgrad
- Erstellt/Geladen am, Gelöst am
- Benötigte Zeit
- "Später wieder"-Flag
- **Subjektive Tags** (frei, z.B. auch Sterne als Tag)
- Verwendete Hilfen (Art + Häufigkeit)
- Fehleranzahl
- Ergebnis: `solved` / `fail` (Challenge-Limit erreicht) / `abandoned`
- Perfekt-Flag (gelöst mit 0 Fehlern, ohne aktive Hints)

Statistiken: Durchschnittszeit, Lösungsquote, Fehlerquote, Perfekt-Quote, Streak, Hilfen-Verlauf

---

## Terminalfenster
- Größenprüfung beim Start
- Zu klein → Hinweis, live auf Resize-Events reagieren (crossterm)
- Spiel pausiert, nahtlos weiter wenn groß genug
- Schriftgröße nicht steuerbar — Hinweis in Doku

---

## Multiplayer

### Host:
- Generiert das Puzzle und verteilt es beim Start an alle Clients
- Vierstelliger Session-Code wird generiert
- LAN-IP und WAN-IP angezeigt (WAN via `api.ipify.org`)
- Hinweis auf Port-Forwarding für WAN
- Lobby-Liste wächst live mit beitretenden Spielern
- Host startet Spiel manuell → dramatischer `3 – 2 – 1` Countdown bei allen Clients sichtbar
- Ab Start: kein Join mehr möglich
- Zu spät gekommene: "Bitte auf nächste Runde warten"

### Client:
- IP, Port, Name und Session-Code eingeben
- Wartet in Lobby bis Host startet

### Während des Spiels:
- Fortschritt der anderen als Ratio sichtbar (z.B. `7/81`)
- Keine aktiven Hints im Multiplayer
- Passive Hilfen verfügbar (gleiche Ziffern hervorheben, ungültige Eingaben markieren) — Notizen-Auto-Update deaktiviert
- Undo, Redo, Zelle leeren verfügbar (inkl. Rückfrage bei Löschen)

### Nach der Runde:
- Sieger wird verkündet
- Live-Scoreboard aktualisiert sich
- Host entscheidet ob auf alle gewartet wird
- Zwischen Runden: neue Spieler können joinen, bestehende aussteigen

### Session-Punkte:
- Kumulativ über mehrere Runden
- Zwei Modi wählbar:
  - `linear` — Konstanz wird belohnt, 2× 2. Platz = 1× 1. + 1× 3.
  - `competitive` — Sieg-Bonus
- Host legt alle Session-Parameter fest (Scoring-Modus, Port, Schwierigkeit etc.)

### Verbindungsabbruch:
- Runde wird abgebrochen, Ergebnis annulliert, zurück zur Lobby
- Host beendet → Session geschlossen, Abschiedsmeldung an alle

---

## i18n
- **Default:** Englisch
- Englisch und Deutsch vordefiniert, Sprachdateien erweiterbar
- **Alle** sichtbaren Texte i18n-fähig: UI-Labels, Bestätigungsdialoge, Fehlermeldungen, Help-Texte, Statusanzeigen
- Wählbar via CLI-Arg: `--lang de`
- Auch im Settings-Screen änderbar
- Config speichert Wahl persistent; CLI-Arg überschreibt einmalig

---

## Meilensteine

### M1 — Solver & Generator
- Solver-Modul vollständig (alle Strategien bis Schwer, Backtracking-Fallback)
- Puzzle-Generator mit Schwierigkeitssteuerung
- Vollständig testbar ohne UI
- **Multiplayer-Vorbereitung:** Game-State als serialisierbares Struct; alle Spielaktionen als Events (Command-Pattern) — Basis für Undo und spätere Netzwerk-Synchronisation

### M2 — Minimale TUI
- Grid-Rendering (Gitter, Farbsystem, vorgegebene vs. eingetragene Ziffern)
- Keyboard-Navigation (Numpad 2-stufig, Pfeiltasten, Eingabe-Modus)
- Zifferneingabe: Lösungs- und Notiz-Modus, Zelle leeren mit Rückfrage
- Undo / Redo
- Erstes eingebautes Ziffern-Stil: `retro`
- **Multiplayer-Vorbereitung:** Timer abstrahiert (nicht direkt `std::time`)

### M3 — Solo vollständig
- SQLite-Datenbank (Original-Puzzle + Spielstand)
- Auto-Save und Weiterspielen
- Settings-Screen mit Press-to-bind Keybinding-Konfiguration
- Help-Screen (dynamisch aus Config generiert)
- Support-System: passive Hilfen (togglebar) + aktive Hints
- Falscheingaben-Feedback (flash / beep / silent)
- Zweiter Ziffern-Stil: `awkward-retro`; Custom-Stil-Loader
- **Multiplayer-Vorbereitung:** Strikte Trennung UI ↔ Spiellogik abgeschlossen

### M4 — Polish
- Maussteuerung (Panel mit Toggle, Ziffern, Undo/Redo/Löschen/Hint/Help/Settings)
- i18n vollständig (alle Texte, Englisch + Deutsch)
- CLI-Args komplett (`--lang`, `-s`, `-f`)
- Vollständige Konfigurierbarkeit (Farben, Keybindings, Stile, Spielverhalten)
- Terminalfenster Resize-Handling

### M5 — Multiplayer
- tokio async Runtime, TCP
- Session-Codes, Lobby, Host/Client-Rollen
- 3-2-1 Countdown beim Start
- Fortschrittsanzeige der Mitspieler
- Scoreboard (linear / competitive)
- Verbindungsabbruch-Behandlung

---

## Offen / Noch nicht entschieden
- `*`-Menü Hooks (3, 5–9): Belegung folgt aus Testläufen