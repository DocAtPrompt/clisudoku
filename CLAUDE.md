# SudokuCLI — Projektregeln für Claude

## Terminal-Farben: nur ANSI-Farben

**Regel:** Ausschließlich die 16 benannten ANSI-Farben aus `crossterm::style::Color` verwenden.  
`Color::Rgb { r, g, b }` und `Color::AnsiValue(n)` sind **verboten**.

**Erlaubt:**
```
Color::Black       Color::DarkGrey
Color::Red         Color::DarkRed
Color::Green       Color::DarkGreen
Color::Yellow      Color::DarkYellow
Color::Blue        Color::DarkBlue
Color::Magenta     Color::DarkMagenta
Color::Cyan        Color::DarkCyan
Color::White       Color::Grey
Color::Reset
```

**Verboten:**
```
Color::Rgb { r, g, b }   // True-Color — funktioniert nicht zuverlässig in allen Terminals
Color::AnsiValue(n)       // 256-Farben — unnötige Komplexität
```

**Warum:** `Color::Rgb` kann in Terminals ohne True-Color-Unterstützung ignoriert werden,
wodurch die Vordergrundfarbe auf dem letzten gesetzten Wert bleibt — oft unsichtbarer
Text auf gleichfarbigem Hintergrund. Dieses Problem ist mehrfach aufgetreten.

**Ausnahmen:** Keine. Bestehende Ausnahmen in `colors.rs` (z. B. `cell_active_box_bg`)
werden beim nächsten Refactoring bereinigt.
