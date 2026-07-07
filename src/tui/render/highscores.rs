use crate::db::ScoreEntry;
use crate::i18n::Strings;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Difficulty-tab keys. These MUST stay in English: they double as the DB
/// filter key (`ScoreEntry::difficulty`, written by `Difficulty::to_db_str`).
/// Display labels are localized separately via `tab_label`.
pub const DIFFICULTY_TABS: &[&str] =
    &["Easy", "Medium", "Hard", "Extreme", "Expert", "Sparse", "All"];

/// Localized display label for a difficulty-tab key.
fn tab_label<'a>(key: &str, strings: &'a Strings) -> &'a str {
    match key {
        "Easy" => strings.difficulty_easy,
        "Medium" => strings.difficulty_medium,
        "Hard" => strings.difficulty_hard,
        "Extreme" => strings.difficulty_extreme,
        "Expert" => strings.difficulty_expert,
        "Sparse" => strings.difficulty_bare_minimum,
        _ => strings.highscores_all, // "All"
    }
}

pub fn render_highscores(
    out: &mut impl Write,
    scores: &[ScoreEntry],
    difficulty_tab: usize,
    colors: &ColorScheme,
    strings: &Strings,
) -> io::Result<()> {
    // Title
    queue!(
        out,
        MoveTo(2, 1),
        SetForegroundColor(colors.digit_given),
        SetBackgroundColor(colors.ui_background),
        Print(strings.highscores_title)
    )?;

    // Tabs
    for (i, tab) in DIFFICULTY_TABS.iter().enumerate() {
        let (fg, bg) = if i == difficulty_tab {
            (colors.ui_cursor_fg, colors.ui_cursor_bg)
        } else {
            (colors.ui_text, colors.ui_background)
        };
        queue!(
            out,
            MoveTo(2 + i as u16 * 12, 3),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(format!(" {} ", tab_label(tab, strings)))
        )?;
    }

    // Filter scores for current tab
    let tab_diff = DIFFICULTY_TABS[difficulty_tab];
    let filtered: Vec<&ScoreEntry> = if tab_diff == "All" {
        scores.iter().collect()
    } else {
        scores.iter().filter(|s| s.difficulty == tab_diff).collect()
    };

    // Header
    queue!(
        out,
        MoveTo(2, 5),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(" # \u{2502} Time  \u{2502} Date       \u{2502} Hints \u{2502} Err \u{2502} Scan \u{2502} Rating")
    )?;
    queue!(
        out,
        MoveTo(2, 6),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print("\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{253c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}")
    )?;

    for (i, score) in filtered.iter().take(10).enumerate() {
        let secs = score.time_ms / 1000;
        let time_str = format!("{:02}:{:02}", secs / 60, secs % 60);
        let date = score.finished_at.get(..10).unwrap_or("");
        let scan = if score.scan_used { "yes" } else { "no" };
        let rating = score.rating.map_or("-".to_string(), |r| r.to_string());
        let row = format!(
            "{:>2} \u{2502} {} \u{2502} {} \u{2502} {:>5} \u{2502} {:>3} \u{2502} {:>4} \u{2502} {:>6}",
            i + 1,
            time_str,
            date,
            score.hint_count,
            score.error_count,
            scan,
            rating
        );
        queue!(
            out,
            MoveTo(2, 7 + i as u16),
            SetForegroundColor(colors.ui_text),
            SetBackgroundColor(colors.ui_background),
            Print(row)
        )?;
    }

    // Footer hint
    queue!(
        out,
        MoveTo(2, 18),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(strings.highscores_footer)
    )?;

    queue!(out, ResetColor)
}
