use crate::i18n::Strings;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

pub struct SaveDialogData {
    pub is_solved: bool,
    pub pending_rating: Option<u8>,
    /// Only present when is_solved=true
    pub time_ms: Option<u64>,
    pub rank: Option<usize>,
    pub total: Option<usize>,
    pub hint_count: Option<u32>,
    pub error_count: Option<u32>,
    pub scan_used: Option<bool>,
}

pub fn render_save_dialog(
    out: &mut impl Write,
    data: &SaveDialogData,
    colors: &ColorScheme,
    strings: &Strings,
) -> io::Result<()> {
    let mut row = 2u16;

    if data.is_solved {
        // Result section — title
        queue!(
            out,
            MoveTo(4, row),
            SetForegroundColor(colors.digit_given),
            SetBackgroundColor(colors.ui_background),
            Print(strings.save_dialog_solved_title)
        )?;
        row += 2;

        if let (Some(ms), Some(rank), Some(total)) = (data.time_ms, data.rank, data.total) {
            let secs = ms / 1000;
            queue!(
                out,
                MoveTo(4, row),
                SetForegroundColor(colors.ui_text),
                SetBackgroundColor(colors.ui_background),
                Print(format!(
                    "Time: {:02}:{:02}  ·  Rank: #{} of {}",
                    secs / 60,
                    secs % 60,
                    rank,
                    total
                ))
            )?;
            row += 1;
        }

        if let (Some(hints), Some(errors), Some(scan)) =
            (data.hint_count, data.error_count, data.scan_used)
        {
            let scan_str = if scan { "yes" } else { "no" };
            queue!(
                out,
                MoveTo(4, row),
                SetForegroundColor(colors.ui_text),
                SetBackgroundColor(colors.ui_background),
                Print(format!(
                    "Hints: {}  ·  Errors: {}  ·  Scan: {}",
                    hints, errors, scan_str
                ))
            )?;
            row += 2;
        }
    } else {
        // Unsaved game: show title
        queue!(
            out,
            MoveTo(4, row),
            SetForegroundColor(colors.digit_given),
            SetBackgroundColor(colors.ui_background),
            Print(strings.save_dialog_title)
        )?;
        row += 2;
    }

    // Save prompt
    queue!(
        out,
        MoveTo(4, row),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(strings.save_dialog_save_prompt)
    )?;
    row += 1;

    // Rating
    let rating_str = data
        .pending_rating
        .map_or("[ ]".to_string(), |r| format!("[{}]", r));
    queue!(
        out,
        MoveTo(4, row),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(format!("{} {}", strings.save_dialog_rating, rating_str))
    )?;
    row += 2;

    // Hint line
    queue!(
        out,
        MoveTo(4, row),
        SetForegroundColor(colors.ui_text_dim),
        SetBackgroundColor(colors.ui_background),
        Print(strings.save_dialog_hint)
    )?;

    queue!(out, ResetColor)
}
