/// Zeigt alle 16 ANSI-Named-Colors von crossterm — funktioniert auf allen Plattformen.
/// Aufruf: cargo run --example colors
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::ExecutableCommand;
use std::io::stdout;

const COLORS: &[(Color, &str)] = &[
    (Color::Black,       "Black      "),
    (Color::DarkGrey,    "DarkGrey   "),
    (Color::DarkRed,     "DarkRed    "),
    (Color::Red,         "Red        "),
    (Color::DarkGreen,   "DarkGreen  "),
    (Color::Green,       "Green      "),
    (Color::DarkYellow,  "DarkYellow "),
    (Color::Yellow,      "Yellow     "),
    (Color::DarkBlue,    "DarkBlue   "),
    (Color::Blue,        "Blue       "),
    (Color::DarkMagenta, "DarkMagenta"),
    (Color::Magenta,     "Magenta    "),
    (Color::DarkCyan,    "DarkCyan   "),
    (Color::Cyan,        "Cyan       "),
    (Color::Grey,        "Grey       "),
    (Color::White,       "White      "),
];

fn main() {
    let mut out = stdout();

    println!("─── 16 ANSI Named Colors (maximale Kompatibilität) ───\n");
    println!("{:<14}  {:^15}  {:^15}", "Name", "auf Schwarz", "auf Weiss");
    println!("{}", "─".repeat(50));

    for (color, name) in COLORS {
        // Vorschau auf schwarzem Hintergrund
        out.execute(SetForegroundColor(*color)).unwrap();
        out.execute(SetBackgroundColor(Color::Black)).unwrap();
        out.execute(Print(format!("  {name}  "))).unwrap();

        out.execute(ResetColor).unwrap();
        out.execute(Print("   ")).unwrap();

        // Vorschau auf weißem Hintergrund
        out.execute(SetForegroundColor(*color)).unwrap();
        out.execute(SetBackgroundColor(Color::White)).unwrap();
        out.execute(Print(format!("  {name}  "))).unwrap();

        out.execute(ResetColor).unwrap();
        println!();
    }

    println!("\n─── Als Hintergrundfarbe ───\n");
    for (color, name) in COLORS {
        out.execute(SetForegroundColor(Color::Black)).unwrap();
        out.execute(SetBackgroundColor(*color)).unwrap();
        out.execute(Print(format!("  {name}  "))).unwrap();
        out.execute(ResetColor).unwrap();
        print!("  ");
    }
    println!();

    out.execute(ResetColor).unwrap();
    println!("\n─── Aktuell im Spiel verwendete Farben ───\n");

    let used = &[
        (Color::Black,      "ui_background"),
        (Color::Grey,       "grid_border / ui_text_dim"),
        (Color::White,      "grid_box / digit_given / ui_text"),
        (Color::DarkGrey,   "grid_cell"),
        (Color::DarkBlue,   "cell_active_bg / ui_cursor_bg"),
        (Color::Cyan,       "digit_user"),
        (Color::Red,        "digit_error"),
        (Color::Yellow,     "digit_highlight / note_highlight"),
        (Color::Green,      "digit_scan"),
        (Color::DarkYellow, "firework particle (dim)"),
        (Color::DarkCyan,   "firework particle (dim)"),
        (Color::Magenta,    "firework particle"),
    ];

    for (color, label) in used {
        out.execute(SetForegroundColor(*color)).unwrap();
        out.execute(SetBackgroundColor(Color::Black)).unwrap();
        out.execute(Print(format!("  ███  "))).unwrap();
        out.execute(ResetColor).unwrap();
        println!(" {label}");
    }

    out.execute(ResetColor).unwrap();
}
