// src/terminal_utils.rs

use crossterm::{self, event, terminal, ExecutableCommand};
use std::io::{self, Write};
use ratatui as tui;
use tui::backend::CrosstermBackend;
use tui::Terminal;


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

    /// Restaura o terminal ao seu estado original
    pub fn cleanup(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        self.terminal.backend_mut().execute(terminal::LeaveAlternateScreen)?;
        self.terminal.backend_mut().execute(event::DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

// Drop implementation for automatic cleanup as fallback
impl<W: Write> Drop for TerminalManager<W> {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
