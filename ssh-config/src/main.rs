mod config;
mod entry;  // This line tells Rust to include the `config.rs` file as a module

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
// TUI
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

// LOGS
use simplelog;
use log;
use ratatui::widgets::ListState;
// ::{trace, debug, info, warn, error};

// THREADS


fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Initialize logging
    simplelog::CombinedLogger::init(vec![
        // Log into file
        simplelog::WriteLogger::new(
            simplelog::LevelFilter::Debug,
            simplelog::Config::default(),
            std::fs::File::create("/tmp/ssh-config.log")?,
        ),
    ])?;

    let home_dir = shellexpand::tilde("~").into_owned();
    let config_path = format!("{}/.ssh/config", home_dir);
    log::debug!("Reading {config_path}");
    let configs = config::read_ssh_config(&config_path);

    match configs {
        Ok(entries) => {
            // Print the whole vector
            //println!("{:?}", entries);

            for entry in &entries {
                log::debug!("\n{}", entry);
                //println!("{}", entry);
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
            log::error!("Failed to read SSH config: {}", e);
        }
    }

    Ok(())
}

enum UIEvent {
    Input(Event),
    UpdateSelection(usize),
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
            widgets::ListItem::new(text::Span::raw(entry.host.clone()))
                .style(Style::default().fg(Color::White))
        })
        .collect();

    let mut list_state = widgets::ListState::default();
    if !hosts.is_empty() {
        list_state.select(Some(0));
    }

    // Create a channel to communicate between the event handler thread and the main thread
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to handle events
    let tx_clone = tx.clone();
    let entries_clone = entries.clone();
    thread::spawn(move || {
        let mut list_state = ListState::default();
        if !entries_clone.is_empty() {
            list_state.select(Some(0));
        }
        loop {
            if event::poll(Duration::from_secs(0)).unwrap() {
                if let Ok(event) = event::read() {
                    match event {
                        Event::Key(key) => {
                            let selected_index = match key.code {
                                KeyCode::Down => {
                                    log::debug!("Down Key pressed!");
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
                                    Some(i)
                                }
                                KeyCode::Up => {
                                    log::debug!("Up Key pressed!");
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
                                    Some(i)
                                }
                                KeyCode::Char('q') => {
                                    log::debug!("'q' Key pressed!");
                                    tx_clone.send(UIEvent::Input(event)).unwrap();
                                    return;
                                }
                                KeyCode::Enter => {
                                    log::debug!("Enter Key pressed!");
                                    log::debug!("list_state.selected() = {:?}", list_state.selected());
                                    if let Some(selected) = list_state.selected() {
                                        log::info!("\n{}", &entries[selected]);
                                    }
                                    None
                                }
                                _ => None,
                            };

                            if let Some(index) = selected_index {
                                tx_clone.send(UIEvent::UpdateSelection(index)).unwrap();
                            }
                        }

                        Event::Mouse(mouse_event) => {
                            if let event::MouseEventKind::Down(_) = mouse_event.kind {
                                let list_start = 2;
                                if mouse_event.row >= list_start && mouse_event.row < list_start + hosts.len() as u16 {
                                    let index = (mouse_event.row - list_start) as usize;
                                    tx_clone.send(UIEvent::UpdateSelection(index)).unwrap();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });

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
                .block(widgets::Block::default()
                    .borders(widgets::Borders::ALL)
                    .title(" SSH Hosts ")
                    .border_style(Style::default().fg(Color::Blue))
                    .title_style(Style::default().fg(Color::Blue))
                )
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;

        // Handle events from the channel
        if let Ok(ui_event) = rx.recv_timeout(Duration::from_secs(0)) {
            match ui_event {
                UIEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('q'), .. })) => break,
                UIEvent::UpdateSelection(index) => {
                    log::debug!("Updating selection to index: {}", index);
                    list_state.select(Some(index));
                }
                _ => {}
        }


        /*
        if event::poll(std::time::Duration::from_secs(0))? {
            if let Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        log::debug!("'q' Key pressed!");
                        break
                    },
                    KeyCode::Down => {
                        log::debug!("Down Key pressed!");
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
                        log::debug!("i = {}", i);
                        log::debug!("Some(i) = {:?}", Some(i));
                        list_state.select(Some(i));
                        log::debug!("list_state = {:?}", list_state.selected());
                    }
                    KeyCode::Up => {
                        log::debug!("Up Key pressed!");
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
                        log::debug!("i = {}", i);
                        log::debug!("Some(i) = {:?}", Some(i));
                        list_state.select(Some(i));
                        log::debug!("list_state = {:?}", list_state.selected());
                    }
                    KeyCode::Enter => {
                        log::debug!("Enter Key pressed");
                        log::debug!("list_state.selected() = {:?}", list_state.selected());
                        if let Some(selected) = list_state.selected() {
                            log::info!("\n{}", &entries[selected]);
                        }
                    }
                    _ => {}
                }
            }

            if let Event::Mouse(mouse_event) = event::read()? {
                match mouse_event.kind {
                    event::MouseEventKind::Down(_) => {
                        // There are 2 rows that doesn't contain elements from the list
                        let list_start = 2;
                        if mouse_event.row >= list_start && mouse_event.row < list_start + hosts.len() as u16 {
                            list_state.select(Some((mouse_event.row - list_start) as usize));
                        }
                        log::debug!("mouse_event.row = {}, list_state.selected() = {:?}", mouse_event.row, list_state.selected());
                    }
                    _ => {}
                }
            }
            */
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
