// src/tui/terminal.rs
use crossterm::{
    cursor, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};

/// RAII guard: enters raw mode + alternate screen on creation,
/// restores terminal state on Drop (even on panic).
pub struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    pub fn setup() -> io::Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        Ok(Self { stdout })
    }

    /// Borrow stdout for rendering.
    pub fn stdout(&mut self) -> &mut Stdout {
        &mut self.stdout
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Restore terminal even if we panic — best effort, ignore errors.
        // Always disable mouse capture so no stray mouse events appear after exit.
        let _ = execute!(
            self.stdout,
            crossterm::event::DisableMouseCapture,
            LeaveAlternateScreen,
            cursor::Show
        );
        let _ = terminal::disable_raw_mode();
    }
}

/// Send the ANSI escape to enable mouse capture in the active terminal.
/// Best-effort — ignores errors so callers can use `let _ = enable_mouse_capture()`.
pub fn enable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), crossterm::event::EnableMouseCapture)
}

/// Send the ANSI escape to disable mouse capture in the active terminal.
pub fn disable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), crossterm::event::DisableMouseCapture)
}
