// src/tui/render/boss.rs
//
// "Boss Key" disguise screen — renders a convincing fake terminal showing the
// user's home directory so the game can be hidden at a glance.
//
// Layout:
//   Last login: <plausible timestamp> on ttys003
//
//   user@host ~ % ls
//   Documents    Downloads    Desktop    Music    Pictures    …
//   user@host ~ % ▋

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::env;
use std::fs;
use std::io::{self, Write};

/// Render the boss-key disguise screen, filling the entire terminal.
pub fn render_boss(out: &mut impl Write) -> io::Result<()> {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));

    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into());
    let hostname = detect_hostname();
    let home = env::var("HOME")
        .unwrap_or_else(|_| format!("/home/{}", username));

    // Visible (non-hidden) home directory entries, sorted.
    let mut entries: Vec<String> = fs::read_dir(&home)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| !n.starts_with('.'))
                .collect()
        })
        .unwrap_or_default();
    entries.sort();

    let prompt = format!("{}@{} ~ % ", username, hostname);

    // ── Fill screen with black ────────────────────────────────────────────────
    let blank = " ".repeat(cols as usize);
    queue!(out, SetBackgroundColor(Color::Black), SetForegroundColor(Color::White))?;
    for r in 0..rows {
        queue!(out, MoveTo(0, r), Print(&blank))?;
    }

    // ── Content ───────────────────────────────────────────────────────────────
    let mut row = 0u16;

    // "Last login" line — approximate real current time for authenticity.
    queue!(out, MoveTo(0, row), Print(last_login_line()))?;
    row += 2; // blank line after

    // ls invocation
    queue!(out,
        MoveTo(0, row),
        SetForegroundColor(Color::White),
        Print(format!("{}{}", prompt, "ls"))
    )?;
    row += 1;

    // Directory listing in columns
    if !entries.is_empty() {
        let max_len = entries.iter().map(|e| e.len()).max().unwrap_or(8);
        let col_w = max_len + 4;
        let num_cols = ((cols as usize) / col_w).max(1);

        for chunk in entries.chunks(num_cols) {
            if row >= rows.saturating_sub(3) {
                break;
            }
            let line: String = chunk
                .iter()
                .map(|e| format!("{:<col_w$}", e, col_w = col_w))
                .collect::<Vec<_>>()
                .join("");
            queue!(out, MoveTo(0, row), Print(line.trim_end()))?;
            row += 1;
        }
    }

    row += 1;

    // New prompt with block cursor
    if row < rows {
        queue!(out,
            MoveTo(0, row),
            SetForegroundColor(Color::White),
            Print(format!("{}▋", prompt))
        )?;
    }

    queue!(out, ResetColor)
}

fn detect_hostname() -> String {
    // Try environment variables first (set on many systems).
    if let Ok(h) = env::var("HOSTNAME") {
        if !h.is_empty() {
            return h;
        }
    }
    // macOS / some Linux: read from /etc/hostname.
    if let Ok(h) = fs::read_to_string("/etc/hostname") {
        let h = h.trim().to_string();
        if !h.is_empty() {
            return h;
        }
    }
    "localhost".into()
}

/// Build a plausible "Last login" string using the real system time.
fn last_login_line() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Simple manual formatting — avoid a chrono dependency.
    // Offset by ~15 minutes to look like the login was a bit earlier.
    let login_secs = secs.saturating_sub(900);
    let days  = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

    // Days since Unix epoch → weekday (Zeller-lite).
    let day_of_week = ((login_secs / 86400 + 4) % 7) as usize; // epoch was Thursday
    // Approximate month/day — good enough for an easter egg.
    let day_of_year = (login_secs % (365 * 86400)) / 86400;
    let month_idx   = (day_of_year / 30).min(11) as usize;
    let day         = (day_of_year % 30) + 1;
    let hour        = (login_secs % 86400) / 3600;
    let min         = (login_secs % 3600) / 60;
    let sec         = login_secs % 60;

    format!(
        "Last login: {} {} {:2} {:02}:{:02}:{:02} on ttys003",
        days[day_of_week], months[month_idx], day, hour, min, sec
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_boss_does_not_panic() {
        let mut buf = Vec::new();
        render_boss(&mut buf).unwrap();
        assert!(!buf.is_empty());
    }

    #[test]
    fn last_login_line_contains_expected_parts() {
        let line = last_login_line();
        assert!(line.starts_with("Last login:"));
        assert!(line.contains("ttys003"));
    }
}
