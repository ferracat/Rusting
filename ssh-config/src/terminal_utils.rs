// src/terminal_utils.rs

use crossterm::{self, event, terminal, ExecutableCommand};
use std::io::{self, Write};
use ratatui as tui;
use tui::backend::CrosstermBackend;
use tui::Terminal;

/// Sets up the terminal in raw mode and switches to the alternate screen.
pub fn setup_terminal<W: Write>(terminal: &mut Terminal<CrosstermBackend<W>>) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    terminal.backend_mut().execute(terminal::EnterAlternateScreen)?;
    terminal.backend_mut().execute(event::EnableMouseCapture)?;
    terminal.hide_cursor()?;
    Ok(())
}

/// Restores the terminal to its normal state by disabling raw mode,
/// leaving the alternate screen, and showing the cursor.
pub fn restore_terminal<W: Write>(terminal: &mut Terminal<CrosstermBackend<W>>) -> io::Result<()> {
    terminal::disable_raw_mode()?;
    terminal.backend_mut().execute(terminal::LeaveAlternateScreen)?;
    terminal.backend_mut().execute(event::DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}



pub struct TerminalManager<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
}

impl<W: Write> TerminalManager<W> {
    /// Creates a new TerminalManager, setting up the terminal in raw mode
    /// and enabling alternate screen and mouse capture.
    pub fn new(mut stdout: W) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;
        stdout.execute(event::EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Provides a `draw` method that allows controlled access to the `terminal` field for rendering
    pub fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut tui::Frame<'_>),
    {
        self.terminal.draw(f).map(|_| ()) // Map the result to `Result<(), Error>`
    }
}

// Drop implementation for automatic cleanup
impl<W: Write> Drop for TerminalManager<W> {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = self.terminal.backend_mut().execute(terminal::LeaveAlternateScreen);
        let _ = self.terminal.backend_mut().execute(event::DisableMouseCapture);
        let _ = self.terminal.show_cursor();
    }
}