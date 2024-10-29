mod config;
mod entry;  // This line tells Rust to include the `config.rs` file as a module
mod app;
use app::AppMode; // Bring AppMode into scope

use std::time::Duration;
use std::process;

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
    widgets::ListState,
    layout,
};

// LOGS
use simplelog;
use log;
// ::{trace, debug, info, warn, error};

// THREADS
use std::sync::{mpsc, Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::thread::sleep;

// SIGNAL
use ctrlc;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Shared flag to indicate unsaved changes
    let has_changes = Arc::new(AtomicBool::new(false));

    // Set up the Ctrl+C handler to check `has_changes` and exit
    let has_changes_clone = Arc::clone(&has_changes);
    ctrlc::set_handler(move || {
        if has_changes_clone.load(Ordering::SeqCst) {
            log::debug!("There are unsaved changes!");
        } else {
            log::debug!("No unsaved changes.");
        }
        process::exit(1);
    }).expect("Error setting Ctrl+C handler");


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
    Exit, // Exit event to stop the program by breaking from the main loop
    Search, // Search event to enter the search mode
    Normal, // Normal event to enter the normal mode
}


// Function to access the list_state mutex in a cleaner way and get the currently selected index
fn get_selected_index(list_state: &Arc<Mutex<widgets::ListState>>) -> Option<usize> {
    if let Ok(list_state_guard) = list_state.lock() {
        list_state_guard.selected()
    } else {
        log::error!("Failed to acquire lock on list_state");
        None
    }
}

// Function to access the list_state mutex in a cleaner way and set the selected index
fn set_selected_index(list_state: &Arc<Mutex<widgets::ListState>>, index: Option<usize>) {
    if let Ok(mut list_state_guard) = list_state.lock() {
        list_state_guard.select(index);
    } else {
        log::error!("Failed to acquire lock on list_state");
    }
}

// Generic function to safely access a value inside an Arc<Mutex<T>>
fn with_mutex<T, R, F>(arc_mutex: &Arc<Mutex<T>>, name: Option<&str>, f: F) -> Option<R>
where
    F: FnOnce(&mut T) -> R,
{
    if let Ok(mut guard) = arc_mutex.lock() {
        Some(f(&mut *guard))
    } else {
        match name {
            Some(name) => log::error!("Failed to acquire lock on {}", name),
            None => log::error!("Failed to acquire lock"),
        }
        None
    }
}



fn run_tui(entries: Vec<entry::SshConfigEntry>) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    // This returns a value. The .unwrap() is used to get it out and ignore it so that the build warning is cleared
    // It can cause the program to panic in case of error.
    //crossterm::terminal::enable_raw_mode().unwrap();
    // To better handle and propagate the result the following is advised -> ?
    terminal::enable_raw_mode()?;

    // Use AppMode within this function
    let mut app_mode = AppMode::Normal;

    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    // Create the list of hostnames and wrap it in Arc and Mutex (Atomic Reference Counted smart pointer with a mutex for safe access across threads)
    let hosts = Arc::new(Mutex::new(
        entries.iter()
            .map(|entry| {
                widgets::ListItem::new(text::Span::raw(entry.host.clone()))
                    .style(Style::default().fg(Color::White))
            })
            .collect::<Vec<widgets::ListItem>>(),
    ));

    // Wrap list_state in an Arc and Mutex for shared access
    let list_state = Arc::new(Mutex::new(widgets::ListState::default()));

    // Clone pointers to `list_state` for the thread and main loop
    let list_state_thread = Arc::clone(&list_state);
    let list_state_main = Arc::clone(&list_state);

    // Use the generic function 'with_mutex' to access the hosts vector
    with_mutex(&hosts, Some("hosts"), |hosts| {
        if !hosts.is_empty() {
            set_selected_index(&list_state, Some(0));
        }
    });

    // Create a channel to communicate between the event handler thread and the main thread
    let (tx, rx) = mpsc::channel();

    // Clone pointers to `hosts` for the thread and main loop
    let hosts_thread = Arc::clone(&hosts);
    let hosts_main = Arc::clone(&hosts);

    // Spawn a thread to handle events
    let tx_clone = tx.clone();
    let entries_clone = entries.clone();

    // --- thread to handle mouse and key events ---------------------------------------------------
    thread::spawn(move || {

        // Use the generic function 'with_mutex' to access the hosts vector
        with_mutex(&hosts_thread, Some("hosts_thread"), |hosts| {
            if !hosts.is_empty() {
                set_selected_index(&list_state_thread, Some(0));
            }
        });

        loop {
            if event::poll(Duration::from_secs(0)).unwrap() {
                if let Ok(event) = event::read() {
                    match event {
                        Event::Key(key) => {
                            let selected_index = {
                                // Lock the mutex to access `hosts` safely
                                let hosts = hosts_thread.lock().unwrap();

                                match key.code {
                                    KeyCode::Down => {
                                        log::debug!("Down Key pressed!");
                                        let i = match get_selected_index(&list_state_thread) {
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
                                        let i = match get_selected_index(&list_state_thread) {
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
                                    KeyCode::Char('/') => {
                                        // Switch to search mode
                                        log::debug!("'/' Key pressed!");
                                        tx_clone.send(UIEvent::Search).unwrap(); // Send a signal to enter search mode
                                        None
                                    }
                                    KeyCode::Esc => {
                                        // Exit search mode and return to normal
                                        log::debug!("'Esc' Key pressed!");
                                        tx_clone.send(UIEvent::Normal).unwrap(); // Send a signal to enter normal mode
                                        None
                                    }
                                    KeyCode::Char('q') => {
                                        log::debug!("'q' Key pressed!");
                                        tx_clone.send(UIEvent::Exit).unwrap(); // Send an exit signal
                                        None
                                    }
                                    KeyCode::Enter => {
                                        log::debug!("Enter Key pressed!");
                                        log::debug!("list_state.selected() = {:?}", get_selected_index(&list_state_thread));
                                        if let Some(selected) = get_selected_index(&list_state_thread) {
                                            log::info!("\n{}", &entries[selected]);
                                        }
                                        None
                                    }
                                    _ => None,
                                }
                            };

                            if let Some(index) = selected_index {
                                tx_clone.send(UIEvent::UpdateSelection(index)).unwrap();
                            }
                        }

                        Event::Mouse(mouse_event) => {
                            if let event::MouseEventKind::Down(_) = mouse_event.kind {
                                let list_start = 2; // because of the window frame

                                // Lock the mutex to access `hosts`
                                let hosts = hosts_thread.lock().unwrap();

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

            // Sleep for a short duration to ease the cpu
            sleep(Duration::from_millis(10));
        }
    });

    // --- Main loop -------------------------------------------------------------------------------
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = layout::Layout::default()
                .direction(layout::Direction::Vertical)
                .margin(1)
                .constraints([layout::Constraint::Percentage(100)].as_ref())
                .split(size);

            // Use of 'with_mutex' to access the *hosts* vector
            with_mutex(&hosts_main, Some("hosts_main"), |hosts| {
                let list = widgets::List::new(hosts.iter().cloned())
                    .block(widgets::Block::default()
                        .borders(widgets::Borders::ALL)
                        .title(" SSH Hosts ")
                        .border_style(Style::default().fg(Color::Blue))
                        .title_style(Style::default().fg(Color::Blue))
                    )
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .highlight_symbol(">> ");

                // Use the generic function 'with_mutex' to access the *list_state*
                with_mutex(&list_state_main, Some("list_state"), |list_state_guard| {
                    f.render_stateful_widget(list, chunks[0], list_state_guard);
                });
            });

        })?;

        // Handle events from the channel
        if let Ok(ui_event) = rx.recv_timeout(Duration::from_secs(0)) {
            match ui_event {
                // UIEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('q'), .. })) => {
                //     break;
                // }
                // UIEvent::Input(Event::Key(KeyEvent { code: KeyCode::Char('/'), .. })) => {

                //     break;
                // }
                UIEvent::UpdateSelection(index) => {
                    log::debug!("Updating selection to index: {}", index);
                    set_selected_index(&list_state_main, Some(index));
                }
                UIEvent::Search => {
                    log::info!("Entering search mode.");
                }
                UIEvent::Normal => {
                    log::info!("Entering normal mode.");
                }
                UIEvent::Exit => {
                    log::info!("Exit signal received, breaking main loop.");
                    break; // Break the loop and exit the program
                }
                _ => {}
            }
        }

        // Sleep for a short duration to ease the cpu
        sleep(Duration::from_millis(10));
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
