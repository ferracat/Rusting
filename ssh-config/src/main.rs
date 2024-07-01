mod config;
mod entry;  // This line tells Rust to include the `config.rs` file as a module

use crossterm;
use crossterm::{event, terminal};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};
use ratatui as tui;
use tui::{
    style::Color,
    style::Style,
    text,
    widgets,
    layout,
};

// fn main() {
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = shellexpand::tilde("~").into_owned();
    let config_path = format!("{}/.ssh/config", home_dir);
    let configs = config::read_ssh_config(&config_path);

    match configs {
        Ok(entries) => {
            // Print the whole vector
            println!("{:?}", entries);

            for entry in &entries {
                println!("{}", entry);
                //println!("{}", entry.display());

                //println!("\nHost: {}", entry.host);
                //// If you want to print options as well:
                //for (key, value) in entry.options {
                //    println!("  {} => {}", key, value);
                //}
                //for comment in entry.comments {
                //    println!("  Comment: {}", comment);
                //}
                //
                //// If there isn't any tag, "None" will be printed
                //// let tag_display = entry.tag.as_ref().map_or("None", String::as_str);
                //// println!("  Tag: {}", tag_display);
                //
                //// Handle printing the tag only if it is not "None"
                //if let Some(tag) = &entry.tag {
                //    println!("  Tag: {}", tag);
                //}
            }

            // Run the TUI after reading and printing the entries
            run_tui(entries)?;
        },
        Err(e) => {
            eprintln!("Failed to read SSH config: {}", e);
        }
    }

    Ok(())
}

fn run_tui(entries: Vec<entry::SshConfigEntry>) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    // This returns a value. The .unwrap() is used to get it out and ignore it so that the build warning is cleared
    // It can cause the program to panic in case of error.
    //crossterm::terminal::enable_raw_mode().unwrap();
    // To better handle and propagate the result the following is advised -> ?
    terminal::enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    // Create the list of hostnames
    let hosts: Vec<widgets::ListItem> = entries
        .iter()
        .map(|entry| {
            widgets::ListItem::new(text::Span::raw(&entry.host))
                .style(Style::default().fg(Color::White))
        })
        .collect();

    let mut list_state = widgets::ListState::default();
    if !hosts.is_empty() {
        list_state.select(Some(0));
    }

    // Main loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = layout::Layout::default()
                .direction(layout::Direction::Vertical)
                .margin(1)
                .constraints([layout::Constraint::Percentage(100)].as_ref())
                .split(size);

            let list = widgets::List::new(hosts.clone())
                .block(widgets::Block::default().borders(widgets::Borders::ALL).title(" SSH Hosts "))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;

        if let Event::Key(key) = crossterm::event::read()? {
            match key {
                KeyEvent { code: KeyCode::Char('q'), .. } => break,
                KeyEvent { code: KeyCode::Down, .. } => {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i >= hosts.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyEvent { code: KeyCode::Up, .. } => {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                hosts.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                _ => {}
            }
        }

        if let crossterm::event::Event::Mouse(mouse_event) = event::read()? {
            match mouse_event {
                crossterm::event::MouseEvent {
                    kind: crossterm::event::MouseEventKind::Down(_),
                    row, ..
                } => {
                    let list_start = 2;
                    if row >= list_start && row < list_start + hosts.len() as u16 {
                        list_state.select(Some((row - list_start) as usize));
                    }
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
