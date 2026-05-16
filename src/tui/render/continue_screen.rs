use crate::db::SaveSummary;
use crate::i18n::Strings;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

pub fn render_continue(
    out: &mut impl Write,
    saves: &[SaveSummary],
    selected: usize,
    colors: &ColorScheme,
    strings: &Strings,
) -> io::Result<()> {
    queue!(
        out,
        MoveTo(2, 1),
        SetForegroundColor(colors.digit_given),
        SetBackgroundColor(colors.ui_background),
        Print(strings.continue_title)
    )?;

    for (i, save) in saves.iter().enumerate() {
        let elapsed_secs = save.elapsed_ms / 1000;
        let time_str = format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60);
        let date_str = save.started_at.get(..10).unwrap_or(&save.started_at);
        let last_str = save.last_saved_at.get(..10).unwrap_or(&save.last_saved_at);
        let label = format!(
            "{}. {:8} · {} · {} · last: {}",
            i + 1,
            save.difficulty,
            time_str,
            date_str,
            last_str
        );
        let (fg, bg) = if i == selected {
            (colors.ui_cursor_fg, colors.ui_cursor_bg)
        } else {
            (colors.ui_text, colors.ui_background)
        };
        queue!(
            out,
            MoveTo(2, 3 + i as u16 * 2),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(&label)
        )?;
    }

    // Footer hint
    queue!(
        out,
        MoveTo(2, 3 + saves.len() as u16 * 2 + 1),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(strings.continue_delete)
    )?;

    queue!(out, ResetColor)
}
