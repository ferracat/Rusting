mod config;
mod entry;  // This line tells Rust to include the `config.rs` file as a module
mod liststate_utils;
use liststate_utils::ListStateManager;
mod terminal_utils;
use terminal_utils::TerminalManager;
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
    style::{Color, Modifier, Style},
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
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;


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
    Exit, // Exit event to stop the program by breaking from the main loop
    Exit_error, // Exit event to stop the program by breaking from the main loop when error occurs
    Search, // Search event to enter the search mode
    Normal, // Normal event to enter the normal mode
    Popup, // Open popup with the content of selected entry
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

    // TODO: Ctrl+C not working --------------------------------------------------------------------
    // Set up signal handling for SIGINT (Ctrl+C)
    let mut signals = Signals::new(&[SIGINT]).expect("Failed to set up signals");

    // Shared flag to indicate unsaved changes
    let has_changes = Arc::new(AtomicBool::new(false));

    // Set up signal handling for SIGINT (Ctrl+C) in a separate thread
    let has_changes_clone = Arc::clone(&has_changes);
    thread::spawn(move || {
        for signal in &mut signals {
            if signal == SIGINT {
                if has_changes_clone.load(Ordering::SeqCst) {
                    log::debug!("Ctrl+C pressed. There are unsaved changes!");
                } else {
                    log::debug!("Ctrl+C pressed. No unsaved changes.");
                }
                println!("Exiting gracefully...");
                process::exit(1);  // Exit immediately with code 1
            }
        }
    });


    // --- Terminal Manager ------------------------------------------------------------------------
    // Instantiate TerminalManager, which automatically sets up the terminal
    let mut terminal_manager = TerminalManager::new(std::io::stdout())?;

    // Use AppMode within this function
    let mut app_mode = AppMode::Normal;

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
    let list_state = Arc::new(Mutex::new(ListStateManager::new()));
    // Clone pointers to `list_state` for the thread and main loop
    let list_state_thread = Arc::clone(&list_state);
    let list_state_main = Arc::clone(&list_state);


    // Clone pointers to `hosts` for the thread and main loop
    let hosts_thread = Arc::clone(&hosts);
    let hosts_main = Arc::clone(&hosts);

    // Create a channel to communicate between the event handler thread and the main thread
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    // Clone the entries and wrap them in Arc for shared acess
    let entries_cloned = entries.clone();
    let entries_thread = Arc::new(entries_cloned);

    // Variable to keep the state of the popup
    let popup_open = Arc::new(AtomicBool::new(false));
    // Clone pointers to `popup_open` for the thread and main loop
    let popup_open_thread = Arc::clone(&popup_open);
    let popup_open_main = Arc::clone(&popup_open);

    // --- Thread to handle mouse and key events ---------------------------------------------------
    thread::spawn(move || {

        // Use the generic function 'with_mutex' to access both hosts and list_state structures to put the selector in the first element of the list
        with_mutex(&hosts_thread, Some("hosts_thread"), |hosts| {
            if !hosts.is_empty() {
                with_mutex(&list_state_thread, Some("list_state"), |lstate| {
                    lstate.select(0);
                });
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
                                        let i = with_mutex(&list_state_thread, Some("list_state:Down_Key"), |lstate| {
                                            lstate.get_index()
                                        })
                                        .map(|index| {
                                            if index >= hosts.len() - 1 {
                                                0
                                            } else {
                                                index + 1
                                            }
                                        });
                                        Some(i)
                                    }
                                    KeyCode::Up => {
                                        log::debug!("Up Key pressed!");
                                        let i = with_mutex(&list_state_thread, Some("list_state:Up_Key"), |lstate| {
                                            lstate.get_index()
                                        })
                                        .map(|index| {
                                            if index == 0 {
                                                hosts.len() - 1
                                            } else {
                                                index - 1
                                            }
                                        });
                                        Some(i)
                                    }
                                    KeyCode::Char('/') => {
                                        // Switch to search mode
                                        log::debug!("'/' Key pressed!");
                                        tx_clone.send(UIEvent::Search).unwrap();  // Send a signal to enter Search mode
                                        None
                                    }
                                    KeyCode::Esc => {
                                        // Exit search mode and return to normal
                                        log::debug!("'Esc' Key pressed!");
                                        tx_clone.send(UIEvent::Normal).unwrap();  // Send a signal to enter Normal mode
                                        None
                                    }
                                    KeyCode::Char('q') => {
                                        log::debug!("'q' Key pressed!");
                                        tx_clone.send(UIEvent::Exit).unwrap();  // Send an Exit signal
                                        None
                                    }
                                    KeyCode::Enter => {
                                        log::debug!("Enter Key pressed!");
                                        with_mutex(&list_state_thread, Some("list_state:Enter"), |lstate| {
                                            log::debug!("list_state.selected() = {:?}", lstate.get_index());
                                            log::info!("\n{}", &entries[lstate.get_index()]);
                                            tx_clone.send(UIEvent::Popup).unwrap();
                                        });
                                        None
                                    }
                                    _ => None,
                                }
                            };

                            if let Some(index) = selected_index {
                                tx_clone.send(UIEvent::UpdateSelection(index.unwrap())).unwrap();
                            }
                        }

                        Event::Mouse(mouse_event) => {
                            if let event::MouseEventKind::Down(_) = mouse_event.kind {
                                let list_start = 2;  // because of the window frame taking 2 lines

                                with_mutex(&hosts_thread, Some("hosts:MouseClick"), |hosts| {
                                    if mouse_event.row >= list_start && mouse_event.row < list_start + hosts.len() as u16 {
                                        with_mutex(&list_state_thread, Some("list_state:MouseClick"), |lstate| {
                                            let index = (mouse_event.row - list_start + lstate.scroll_offset as u16 ) as usize;
                                            tx_clone.send(UIEvent::UpdateSelection(index)).unwrap();
                                            log::debug!("index = {:?}, scroll offset = {:?}", lstate.get_index(), lstate.scroll_offset);
                                        });
                                    }
                                });
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
        terminal_manager.draw(|f| {
            let size = f.size();

            let chunks = layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .margin(1)
            .constraints([layout::Constraint::Percentage(100)].as_ref())
            .split(size);

            with_mutex(&list_state_main, Some("list_state"), |lstate| {
                lstate.max_display_items = chunks[0].height as usize - 3; // 3 is the lines ocuppied by the borders
            });

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

                // Use the nested 'with_mutex' to access the *list_state*
                with_mutex(&list_state_main, Some("list_state_main"), |lstate| {
                    f.render_stateful_widget(list, chunks[0], lstate.list_state());
                });
            });

            if popup_open_main.load(Ordering::SeqCst) {

                // Render the popup
                let popup_area = layout::Rect::new(
                    size.width / 4,
                    size.height / 4,
                    size.width / 2,
                    size.height / 2,
                );

                // Display the selected entry or some other content in the popup
                with_mutex(&list_state_main, Some("list_state:render_text_box"), |lstate| {

                    let popup_block = widgets::Block::default()
                        .title(format!(" {} ", entries_thread[lstate.get_index()].host))
                        .borders(widgets::Borders::ALL)
                        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        .style(Style::default().bg(Color::Black));

                    let entry_text = entries_thread[lstate.get_index()].to_string();

                    let text_widget = widgets::Paragraph::new(text::Span::raw(entry_text))
                        .block(popup_block)
                        .wrap(widgets::Wrap { trim: true });

                    f.render_widget(text_widget, popup_area);
                });
            }
        })?;

        // Handle events from the channel
        if let Ok(ui_event) = rx.recv_timeout(Duration::from_secs(0)) {
            match ui_event {
                UIEvent::UpdateSelection(index) => {
                    log::debug!("Updating selection to index: {}", index);
                    with_mutex(&list_state_main, Some("list_state_main"), |lstate| {
                        lstate.select(index);
                    });
                }
                UIEvent::Popup => {
                    log::info!("Open a popup with the entry.");
                    if !popup_open_main.load(Ordering::SeqCst) {
                        popup_open_main.store(true, Ordering::SeqCst);
                    }
                }
                UIEvent::Search => {
                    log::info!("Entering search mode.");
                }
                UIEvent::Normal => {
                    log::info!("Entering normal mode.");
                    if popup_open_main.load(Ordering::SeqCst) {
                        popup_open_main.store(false, Ordering::SeqCst);
                    }
                }
                UIEvent::Exit => {
                    log::info!("Exit signal received, breaking main loop.");
                    break; // Break the loop and exit the program
                }
                UIEvent::Exit_error => {
                    log::info!("Exit signal received with error code.");
                    process::exit(1);
                    break; // Break the loop and exit the program
                }
                _ => {}
            }
        }

        // Sleep for a short duration to ease the cpu load
        sleep(Duration::from_millis(10));
    }

    // > The restore terminal will be done by the drop in **TerminalManager**

    Ok(())
}
