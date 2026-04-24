# Easter Eggs — Ideensammlung

> Backlog für zukünftige Implementierung. Keine feste Reihenfolge.

---

## Bereits implementiert

### Boss Key (`b`)
Versteckt das Spiel hinter einem überzeugenden Fake-Terminal.
- `b` → zeigt echtes Home-Verzeichnis als `ls`-Output mit echtem `user@hostname`-Prompt
- Timer eingefroren
- `b` → zurück zum Spiel
- `Esc` → sofortiger Exit (später: Spielstand speichern und lautlos beenden)

---

## Geplant

### `iddqd` — Cheat-Code (Doom-Referenz)
Eingabe der Zeichenfolge `i`, `d`, `d`, `q`, `d` im Spielmodus füllt alle leeren Zellen
sofort mit der korrekten Lösung. Klassische Doom God-Mode-Referenz.
- Kurze visuelle Bestätigung (z.B. kurzes Aufblinken des Grids)
- Kein Undo möglich nach Aktivierung

### `idkfa` — Notizen-Autofill (Doom-Referenz)
Eingabe von `i`, `d`, `k`, `f`, `a` setzt in allen offenen Zellen korrekte Notizen.
Weniger drastisch als `iddqd` — hilft beim Weiterspielen ohne zu spoilern.

### Konami Code (↑↑↓↓←→←→)
Kurze visuelle Animationssequenz: alle Ziffern im Grid "fallen" nach unten und bauen
sich von oben neu auf. Rein dekorativ, kein Gameplay-Effekt.

### Matrix-Modus
Toggle (ähnlich Boss Key): Ziffern rieseln als grüne Kaskade durch das Grid.
Bleibt spielbar — nur die Darstellung wechselt.

### `xyzzy` (Adventure-Referenz)
Subtile, nie erklärte Reaktion auf die klassische Zork-Beschwörung.
Idee: ein einzelnes Zeichen irgendwo im Grid blinkt einmal auf und verschwindet.
Kein Hinweis, keine Erklärung.

### `sudo`-Eingabe
Antwort im Panel oder als kurze Overlay-Meldung:
`user is not in the sudoers file. This incident will be reported.`

### `help`
Antwort: `This is not a text adventure.`

### Abschluss-Fanfare
Wenn ein Rätsel korrekt gelöst wird: kurze ASCII-Animation statt stillem Abschluss.
Idee: Ziffern blinken kurz auf, danach eine Zeile wie `Solved in 12:34` mit Rahmen.

---

## Noch offen / zu diskutieren

- Sollen Cheat-Codes (`iddqd`, `idkfa`) den Timer stoppen oder eine Strafe hinzufügen?
- Sollen Easter Eggs in der Highscore-Liste markiert werden?
- Freie Tastenbelegung für Boss Key (für M3 konfigurierbar)?
