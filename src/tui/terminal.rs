// src/tui/terminal.rs
use crossterm::{
    cursor,
    execute,
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
        let _ = execute!(self.stdout, LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}
