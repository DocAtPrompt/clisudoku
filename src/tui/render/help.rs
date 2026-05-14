// src/tui/render/help.rs
use crate::i18n::Strings;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write};

pub fn render_help(
    out: &mut impl Write,
    section: usize,
    colors: &ColorScheme,
    strings: &Strings,
) -> io::Result<()> {
    let (cols, rows) = terminal::size().unwrap_or((117, 39));
    let width = (cols as usize).min(117);
    let inner = width.saturating_sub(2);

    let bg = colors.ui_background;
    let fg = colors.ui_text;
    let dim = colors.ui_text_dim;
    let tab_bg = colors.ui_cursor_bg;
    let tab_fg = colors.ui_cursor_fg;

    // ── Title bar ─────────────────────────────────────────────────────────────
    let title = strings.help_title;
    let title_len = title.chars().count();
    let title_pad = inner.saturating_sub(title_len) / 2;
    queue!(
        out,
        MoveTo(0, 0),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(width.saturating_sub(2)))),
        MoveTo(0, 1),
        Print(format!(
            "\u{2551}{}{}{}\u{2551}",
            " ".repeat(title_pad),
            title,
            " ".repeat(inner.saturating_sub(title_pad + title_len))
        )),
        MoveTo(0, 2),
        Print(format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(width.saturating_sub(2)))),
    )?;

    // ── Tab bar ───────────────────────────────────────────────────────────────
    let tabs = [
        strings.help_section_controls,
        strings.help_section_rules,
        strings.help_section_colors,
    ];
    queue!(out, MoveTo(0, 3), SetBackgroundColor(bg), Print("\u{2551}"))?;
    let mut tab_used: usize = 1; // leading ║
    for (i, tab) in tabs.iter().enumerate() {
        let tab_len = tab.chars().count();
        if i == section {
            queue!(
                out,
                SetBackgroundColor(tab_bg),
                SetForegroundColor(tab_fg),
                Print(format!(" [ {} ] ", tab)),
                SetBackgroundColor(bg),
                SetForegroundColor(dim),
            )?;
            tab_used += tab_len + 7;
        } else {
            queue!(
                out,
                SetBackgroundColor(bg),
                SetForegroundColor(dim),
                Print(format!("  {}  ", tab)),
            )?;
            tab_used += tab_len + 4;
        }
    }
    let tab_pad = width.saturating_sub(tab_used + 1);
    queue!(
        out,
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(" ".repeat(tab_pad)),
        Print("\u{2551}"),
        MoveTo(0, 4),
        Print(format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(width.saturating_sub(2)))),
    )?;

    // ── Content ───────────────────────────────────────────────────────────────
    let content_start_row: u16 = 5;
    let available = (rows as usize).saturating_sub(8);

    if section == 2 {
        render_colors_section(out, strings, colors, bg, fg, width, content_start_row, available)?;
    } else {
        let lines: Vec<String> = if section == 0 {
            render_controls_lines(strings)
        } else {
            render_rules_lines(strings)
        };
        for (i, line) in lines.iter().take(available).enumerate() {
            let row = content_start_row + i as u16;
            let line_chars = line.chars().count();
            let pad = width.saturating_sub(3 + line_chars + 1);
            queue!(
                out,
                MoveTo(0, row),
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                Print("\u{2551} "),
                Print(line),
                Print(" ".repeat(pad)),
                Print("\u{2551}"),
            )?;
        }
        for i in lines.len()..available {
            queue!(
                out,
                MoveTo(0, content_start_row + i as u16),
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                Print(format!("\u{2551}{}\u{2551}", " ".repeat(width.saturating_sub(2)))),
            )?;
        }
    }

    // ── Bottom bar ────────────────────────────────────────────────────────────
    let bottom_row = content_start_row + available as u16;
    let hint = strings.help_close_hint;
    let hint_len = hint.chars().count();
    let hint_pad = width.saturating_sub(2 + hint_len + 1);
    queue!(
        out,
        MoveTo(0, bottom_row),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(width.saturating_sub(2)))),
        MoveTo(0, bottom_row + 1),
        SetForegroundColor(dim),
        Print("\u{2551} "),
        Print(hint),
        Print(" ".repeat(hint_pad)),
        SetForegroundColor(fg),
        Print("\u{2551}"),
        MoveTo(0, bottom_row + 2),
        Print(format!("\u{255a}{}\u{255d}", "\u{2550}".repeat(width.saturating_sub(2)))),
        ResetColor,
    )?;
    Ok(())
}

// ── Section 0: Controls ───────────────────────────────────────────────────────

fn render_controls_lines(strings: &Strings) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    v.push(String::new());
    push_group(&mut v, strings.help_group_navigation);
    v.push(strings.ctrl_move.to_string());
    v.push(strings.ctrl_goto.to_string());
    v.push(String::new());
    push_group(&mut v, strings.help_group_quick_nav);
    for line in strings.help_quick_nav_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    push_group(&mut v, strings.help_group_input);
    v.push(strings.ctrl_digit.to_string());
    v.push(strings.ctrl_mode.to_string());
    v.push(strings.ctrl_clear.to_string());
    v.push(strings.ctrl_undo.to_string());
    v.push(strings.ctrl_redo.to_string());
    v.push(String::new());
    push_group(&mut v, strings.help_group_functions);
    v.push(strings.ctrl_hint.to_string());
    v.push(strings.ctrl_scan.to_string());
    v.push(strings.ctrl_errors.to_string());
    v.push(strings.ctrl_pause.to_string());
    v.push(strings.ctrl_mouse.to_string());
    v.push(strings.ctrl_boss.to_string());
    v.push(strings.ctrl_quit.to_string());
    v.push(strings.ctrl_help.to_string());
    v.push(String::new());
    v
}

// ── Section 1: Rules ──────────────────────────────────────────────────────────

fn render_rules_lines(strings: &Strings) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    v.push(String::new());
    push_group(&mut v, strings.help_group_rules);
    for line in strings.help_rules_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    push_group(&mut v, strings.help_group_notes);
    for line in strings.help_notes_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    push_group(&mut v, strings.help_group_hints);
    for line in strings.help_hints_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    v
}

// ── Section 2: Colors ─────────────────────────────────────────────────────────

fn render_colors_section(
    out: &mut impl Write,
    strings: &Strings,
    colors: &ColorScheme,
    bg: Color,
    fg: Color,
    width: usize,
    start_row: u16,
    available: usize,
) -> io::Result<()> {
    // Cell/digit swatches (pairs), then blank line, then hint swatches (one per row).
    type Swatch<'a> = (Color, Color, char, &'a str);
    let cell: &[Swatch] = &[
        (colors.digit_given,        colors.cell_normal_bg,       '\u{2588}', strings.help_color_given),
        (colors.digit_user,         colors.cell_normal_bg,       '\u{2588}', strings.help_color_user),
        (colors.digit_error,        colors.cell_normal_bg,       '\u{2588}', strings.help_color_error),
        (colors.ui_cursor_fg,       colors.ui_cursor_bg,         '\u{2588}', strings.help_color_cursor),
        (colors.digit_user,         colors.cell_active_cross_bg, '\u{2588}', strings.help_color_cross),
        (colors.digit_user,         colors.cell_active_box_bg,   '\u{2588}', strings.help_color_box),
        (colors.digit_scan,         colors.cell_normal_bg,       '\u{2588}', strings.help_color_scan),
        (colors.digit_user,         colors.hover_bg,             '\u{2588}', strings.help_color_hover),
    ];
    // Hint-related: border colors (▐) and fill target (█) — shown individually.
    let hint: &[Swatch] = &[
        (colors.hint_cause_border,  colors.cell_normal_bg,  '\u{2590}', strings.help_color_hint_cause),
        (colors.hint_elim_border,   colors.cell_normal_bg,  '\u{2590}', strings.help_color_hint_elim),
        (colors.digit_user,         colors.hint_target_bg,  '\u{2588}', strings.help_color_hint_target),
    ];

    let mut row_idx: usize = 0;

    // Blank opener
    blank_row(out, bg, fg, width, start_row + row_idx as u16)?;
    row_idx += 1;

    // Cell swatches in pairs
    let mut i = 0;
    while i < cell.len() && row_idx < available {
        render_swatch_pair(out, bg, fg, width, start_row + row_idx as u16,
                           cell[i], if i + 1 < cell.len() { Some(cell[i + 1]) } else { None })?;
        row_idx += 1;
        i += 2;
    }

    // Blank separator before hint colors
    if row_idx < available {
        blank_row(out, bg, fg, width, start_row + row_idx as u16)?;
        row_idx += 1;
    }

    // Hint swatches — one per row (full width)
    for &(sfg, sbg, ch, label) in hint {
        if row_idx >= available { break; }
        let label_len = label.chars().count();
        let pad = width.saturating_sub(3 + 1 + 2 + label_len + 1);
        queue!(
            out,
            MoveTo(0, start_row + row_idx as u16),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print("\u{2551}  "),
            SetForegroundColor(sfg),
            SetBackgroundColor(sbg),
            Print(ch),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(format!("  {}", label)),
            Print(" ".repeat(pad)),
            Print("\u{2551}"),
        )?;
        row_idx += 1;
    }

    while row_idx < available {
        blank_row(out, bg, fg, width, start_row + row_idx as u16)?;
        row_idx += 1;
    }
    Ok(())
}

fn render_swatch_pair(
    out: &mut impl Write,
    bg: Color,
    fg: Color,
    width: usize,
    row: u16,
    left: (Color, Color, char, &str),
    right: Option<(Color, Color, char, &str)>,
) -> io::Result<()> {
    let (fg0, bg0, ch0, label0) = left;
    queue!(
        out,
        MoveTo(0, row),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print("\u{2551}  "),
        SetForegroundColor(fg0),
        SetBackgroundColor(bg0),
        Print(ch0),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(format!("  {:<20}", label0)),
    )?;
    if let Some((fg1, bg1, ch1, label1)) = right {
        let used = 3 + 1 + 2 + 20 + 1 + 2 + label1.chars().count() + 1;
        queue!(
            out,
            SetForegroundColor(fg1),
            SetBackgroundColor(bg1),
            Print(ch1),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(format!("  {}", label1)),
            Print(" ".repeat(width.saturating_sub(used))),
            Print("\u{2551}"),
        )?;
    } else {
        let used = 3 + 1 + 2 + 20;
        queue!(
            out,
            Print(" ".repeat(width.saturating_sub(used + 1))),
            Print("\u{2551}"),
        )?;
    }
    Ok(())
}

fn blank_row(out: &mut impl Write, bg: Color, fg: Color, width: usize, row: u16) -> io::Result<()> {
    queue!(
        out,
        MoveTo(0, row),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(format!("\u{2551}{}\u{2551}", " ".repeat(width.saturating_sub(2)))),
    )
}

fn push_group(v: &mut Vec<String>, label: &str) {
    let label_len = label.chars().count();
    let dashes = 40_usize.saturating_sub(label_len + 4);
    v.push(format!("  \u{2500}\u{2500} {} {}", label, "\u{2500}".repeat(dashes)));
}
