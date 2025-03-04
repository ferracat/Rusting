mod config;
mod entry;  // This line tells Rust to include the `config.rs` file as a module
mod liststate_utils;
use liststate_utils::ListStateManager;
mod terminal_utils;
use terminal_utils::TerminalManager;
mod tui_utils;
use tui_utils::render_popup_table;
mod app;
use app::AppMode;  // Bring AppMode into scope

use std::process;
use std::time::Duration;

// TUI
use crossterm;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui as tui;
use tui::{
    layout,
    style::{Color, Style, Modifier},
    text::{Span, Line},
    widgets::{self, Table, Row, Cell},
    widgets::Block,
};

// LOGS
use log;
use simplelog;
// ::{trace, debug, info, warn, error};

// THREADS
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
    Arc,
    Mutex,
};
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
    Exit,       // Exit event to stop the program by breaking from the main loop
    ExitError,  // Exit event to stop the program by breaking from the main loop when error occurs
    Search,     // Search event to enter the search mode
    Normal,     // Normal event to enter the normal mode
    Popup,      // Open popup with the content of selected entry
    Help,       // Show help popup
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
    // Set up signal handling for SIGINT (Ctrl+C)
    let mut signals = Signals::new(&[SIGINT]).expect("Failed to set up signals");
    
    // Canal para comunicar o sinal SIGINT entre as threads
    let (sigint_tx, sigint_rx) = mpsc::channel();
    let sigint_tx_clone = sigint_tx.clone();

    // Shared flag to indicate unsaved changes
    let has_changes = Arc::new(AtomicBool::new(false));
    let has_changes_clone = Arc::clone(&has_changes);

    // Thread para lidar com sinais
    thread::spawn(move || {
        for signal in signals.forever() {
            if signal == SIGINT {
                if has_changes_clone.load(Ordering::SeqCst) {
                    log::debug!("Ctrl+C pressed. There are unsaved changes!");
                } else {
                    log::debug!("Ctrl+C pressed. No unsaved changes.");
                }
                // Enviar sinal para a thread principal
                let _ = sigint_tx_clone.send(());
                break;
            }
        }
    });

    // --- Terminal Manager ------------------------------------------------------------------------
    // Instantiate TerminalManager, which automatically sets up the terminal
    let mut terminal_manager = TerminalManager::new(std::io::stdout())?;

    // Criar o app_mode como Arc<Mutex>
    let app_mode = Arc::new(Mutex::new(AppMode::Normal));
    let app_mode_thread = Arc::clone(&app_mode);

    // Create the list of hostnames and wrap it in Arc and Mutex (Atomic Reference Counted smart pointer with a mutex for safe access across threads)
    let hosts = Arc::new(Mutex::new(
        entries.iter()
            .map(|entry| {
                widgets::ListItem::new(Span::raw(entry.host.clone()))
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

    // Clone the entries and wrap them in Arc for shared access
    let entries_cloned = entries.clone();
    let entries_thread = Arc::new(entries_cloned);
    let entries_main = Arc::clone(&entries_thread);  // Clone para o loop principal

    // Variable to keep the state of the popup
    let popup_open = Arc::new(AtomicBool::new(false));
    // Clone pointers to `popup_open` for the thread and main loop
    //let popup_open_thread = Arc::clone(&popup_open);
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
                            match key.code {
                                KeyCode::Down => {
                                    log::debug!("Down Key pressed!");
                                    let i = with_mutex(&list_state_thread, Some("list_state:Down_Key"), |lstate| {
                                        lstate.get_index()
                                    })
                                    .map(|index| {
                                        if index >= hosts_thread.lock().unwrap().len() - 1 {
                                            0
                                        } else {
                                            index + 1
                                        }
                                    });
                                    if let Some(i) = i {
                                        tx_clone.send(UIEvent::UpdateSelection(i)).unwrap();
                                    }
                                }
                                KeyCode::Up => {
                                    log::debug!("Up Key pressed!");
                                    let i = with_mutex(&list_state_thread, Some("list_state:Up_Key"), |lstate| {
                                        lstate.get_index()
                                    })
                                    .map(|index| {
                                        if index == 0 {
                                            hosts_thread.lock().unwrap().len() - 1
                                        } else {
                                            index - 1
                                        }
                                    });
                                    if let Some(i) = i {
                                        tx_clone.send(UIEvent::UpdateSelection(i)).unwrap();
                                    }
                                }
                                KeyCode::Char('/') => {
                                    log::debug!("'/' Key pressed!");
                                    tx_clone.send(UIEvent::Search).unwrap();
                                }
                                KeyCode::Esc => {
                                    log::debug!("'Esc' Key pressed!");
                                    tx_clone.send(UIEvent::Normal).unwrap();
                                }
                                KeyCode::Char('q') => {
                                    log::debug!("'q' Key pressed!");
                                    // Verificar se estamos no modo de busca
                                    let is_search = with_mutex(&app_mode_thread, Some("app_mode"), |mode| {
                                        mode.is_search()
                                    }).unwrap_or(false);

                                    if !is_search {
                                        tx_clone.send(UIEvent::Exit).unwrap();
                                    } else {
                                        // Se estiver no modo de busca, deixar o handle_search_mode tratar
                                        handle_search_mode(event, &app_mode_thread, &entries_thread)
                                            .map(|e| tx_clone.send(e).unwrap());
                                    }
                                }
                                KeyCode::Char('h') => {
                                    log::debug!("'h' Key pressed!");
                                    // Verificar se estamos no modo de busca
                                    let is_search = with_mutex(&app_mode_thread, Some("app_mode"), |mode| {
                                        mode.is_search()
                                    }).unwrap_or(false);

                                    if !is_search {
                                        tx_clone.send(UIEvent::Help).unwrap();
                                    } else {
                                        // Se estiver no modo de busca, deixar o handle_search_mode tratar
                                        handle_search_mode(event, &app_mode_thread, &entries_thread)
                                            .map(|e| tx_clone.send(e).unwrap());
                                    }
                                }
                                KeyCode::Enter => {
                                    log::debug!("Enter Key pressed!");
                                    with_mutex(&list_state_thread, Some("list_state:Enter"), |lstate| {
                                        log::debug!("list_state.selected() = {:?}", lstate.get_index());
                                        // Verificar se estamos no modo de busca
                                        with_mutex(&app_mode_thread, Some("app_mode"), |mode| {
                                            if let AppMode::Search { matches, .. } = mode {
                                                // Se estiver no modo de busca, usar o índice dos matches
                                                if !matches.is_empty() {
                                                    let selected = lstate.get_index();
                                                    if selected < matches.len() {
                                                        // Usar o índice real do entry a partir dos matches
                                                        lstate.select(matches[selected]);
                                                    }
                                                }
                                            }
                                        });
                                        tx_clone.send(UIEvent::Popup).unwrap();
                                    });
                                }
                                _ => {
                                    // Tratar outros caracteres quando estiver no modo de busca
                                    handle_search_mode(event, &app_mode_thread, &entries_thread)
                                        .map(|e| tx_clone.send(e).unwrap());
                                }
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
        // Verificar se recebemos um sinal SIGINT
        if sigint_rx.try_recv().is_ok() {
            log::info!("SIGINT received, exiting gracefully...");
            // Restaurar o terminal antes de sair
            terminal_manager.cleanup()?;
            process::exit(0);
        }

        terminal_manager.draw(|f| {
            let size = f.size();

            // Criar um layout com espaço para a barra de pesquisa na parte inferior
            let chunks = layout::Layout::default()
                .direction(layout::Direction::Vertical)
                .margin(1)
                .constraints([
                    layout::Constraint::Min(3),     // Lista principal
                    layout::Constraint::Length(3),  // Barra de pesquisa
                ].as_ref())
                .split(size);

            // Primeiro renderiza a lista
            if !popup_open_main.load(Ordering::SeqCst) {
                with_mutex(&hosts_main, Some("hosts_main"), |hosts| {
                    with_mutex(&app_mode, Some("app_mode"), |mode: &mut AppMode| {
                        // Filtrar a lista se estiver no modo de busca
                        let items_to_show = if let AppMode::Search { matches, query, .. } = mode {
                            log::debug!("Current search: '{}' with {} matches", query, matches.len());
                            if query.is_empty() {
                                hosts.clone()
                            } else {
                                matches.iter()
                                    .map(|&idx| {
                                        // Criar um novo ListItem sem os 3 espaços
                                        let host = &entries_main[idx].host;
                                        widgets::ListItem::new(Span::raw(host.to_string()))
                                    })
                                    .collect::<Vec<_>>()
                            }
                        } else {
                            hosts.clone()
                        };

                        let items_clone = items_to_show.clone();  // Clone para usar depois

                        let list = widgets::List::new(items_to_show)
                            .block(
                                Block::default()
                                    .borders(widgets::Borders::ALL)
                                    .border_style(Style::default().fg(Color::Blue))
                                    .title(" SSH Hosts ")
                                    .title_style(Style::default().fg(Color::Blue)),
                            )
                            .highlight_symbol(">> ")
                            .highlight_style(Style::default().fg(Color::Yellow));

                        with_mutex(&list_state_main, Some("list_state_main"), |lstate| {
                            // Resetar a seleção se não houver itens
                            if items_clone.is_empty() {
                                lstate.select(0);
                            }
                            f.render_stateful_widget(list, chunks[0], lstate.list_state());
                        });
                    });
                });
            }

            // Depois renderiza a barra de pesquisa na parte inferior
            with_mutex(&app_mode, Some("app_mode"), |mode: &mut AppMode| {
                if mode.is_search() {
                    tui_utils::render_search_bar(f, chunks[1], mode);
                }
            });

            // Atualiza o número máximo de itens visíveis
            with_mutex(&list_state_main, Some("list_state"), |lstate| {
                lstate.max_display_items = chunks[0].height as usize - 3;
            });

            // Renderiza o popup se necessário
            if popup_open_main.load(Ordering::SeqCst) {
                let popup_area = layout::Rect::new(
                    size.width / 6,
                    size.height / 6,
                    4 * size.width / 6,
                    4 * size.height / 6,
                );

                with_mutex(&app_mode, Some("app_mode"), |mode| {
                    match mode {
                        AppMode::Help => {
                            // Definir os dados da tabela
                            let rows = vec![
                                Row::new(vec![Cell::from(""), Cell::from("")]),  // Linha em branco
                                Row::new(vec![
                                    Cell::from(Span::styled("  h", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
                                    Cell::from("This menu")
                                ]),
                                Row::new(vec![
                                    Cell::from(Span::styled("  q", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
                                    Cell::from("Quit")
                                ]),
                                Row::new(vec![
                                    Cell::from(Span::styled("  ESC", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
                                    Cell::from("Back to normal mode")
                                ]),
                                Row::new(vec![
                                    Cell::from(Span::styled("  /", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
                                    Cell::from("Search mode")
                                ]),
                                Row::new(vec![
                                    Cell::from(Span::styled("  e", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
                                    Cell::from("Edit mode")
                                ]),
                            ];
                            
                            // Criar layout vertical para título e tabela
                            let help_layout = layout::Layout::default()
                                .direction(layout::Direction::Vertical)
                                .constraints([
                                    layout::Constraint::Length(2),  // Espaço para o título
                                    layout::Constraint::Min(10),    // Espaço para a tabela
                                ])
                                .split(popup_area);

                            // Renderizar o título centralizado
                            let title = widgets::Paragraph::new(
                                Span::styled(" Available Commands ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                            )
                                .style(Style::default())
                                .alignment(layout::Alignment::Center);
                            f.render_widget(title, help_layout[0]);

                            // Renderizar a tabela
                            let help_block = Block::default()
                                .borders(widgets::Borders::ALL)
                                .border_style(Style::default().fg(Color::Blue));

                            let table = Table::new(
                                rows,
                                &[
                                    layout::Constraint::Length(6),  // Largura fixa para comandos (aumentada de 4 para 6)
                                    layout::Constraint::Min(20),    // Resto do espaço para descrições
                                ]
                            )
                                .block(help_block)
                                .style(Style::default())
                                .column_spacing(1);                 // Espaço entre colunas

                            f.render_widget(table, help_layout[1]);
                        },
                        _ => {
                            with_mutex(&list_state_main, Some("list_state:render_text_box"), |lstate| {
                                let entry = &entries_main[lstate.get_index()];
                                render_popup_table(f, popup_area, entry);
                            });
                        }
                    }
                });
            }
        })?;

        // Handle events from the channel
        if let Ok(ui_event) = rx.recv_timeout(Duration::from_millis(10)) {
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
                    with_mutex(&app_mode, Some("app_mode"), |mode: &mut AppMode| {
                        log::debug!("Initializing search mode");
                        *mode = AppMode::Search {
                            query: String::new(),
                            cursor_position: 0,
                            matches: (0..entries.len()).collect(), // Inicialmente, todos os itens são matches
                            current_match: None,
                        };
                    });
                }
                UIEvent::Normal => {
                    log::info!("Entering normal mode.");
                    with_mutex(&app_mode, Some("app_mode"), |mode: &mut AppMode| {
                        *mode = AppMode::Normal;
                    });
                    if popup_open_main.load(Ordering::SeqCst) {
                        popup_open_main.store(false, Ordering::SeqCst);
                    }
                }
                UIEvent::Exit => {
                    // Só permitir sair com 'q' se não estiver no modo de busca
                    let should_exit = with_mutex(&app_mode, Some("app_mode"), |mode| {
                        !mode.is_search()
                    }).unwrap_or(true);

                    if should_exit {
                        log::info!("Exit signal received, breaking main loop.");
                        break;
                    } else {
                        log::debug!("Exit signal ignored in search mode");
                    }
                }
                UIEvent::ExitError => {
                    log::info!("Exit signal received with error code.");
                    break;
                }
                UIEvent::Help => {
                    log::info!("Showing help popup.");
                    with_mutex(&app_mode, Some("app_mode"), |mode| {
                        *mode = AppMode::Help;
                    });
                    if !popup_open_main.load(Ordering::SeqCst) {
                        popup_open_main.store(true, Ordering::SeqCst);
                    }
                }
                _ => {}
            }
        }
    }

    // Restaurar o terminal antes de sair
    terminal_manager.cleanup()?;

    Ok(())
}



#[derive(Debug)]
enum NavigationDirection {
    Up,
    Down,
}

fn handle_navigation(
    list_state: &Arc<Mutex<ListStateManager>>,
    hosts: &Arc<Mutex<Vec<widgets::ListItem>>>,
    direction: NavigationDirection,
) -> usize {
    with_mutex(list_state, Some("list_state"), |lstate| {
        let current_index = lstate.get_index(); // Directly get the current index as usize
        let total_hosts = hosts.lock().unwrap().len();

        // Compute the new index based on navigation direction
        let new_index = match direction {
            NavigationDirection::Down => {
                if current_index >= total_hosts - 1 {
                    0 // Wrap around to the start
                } else {
                    current_index + 1
                }
            }
            NavigationDirection::Up => {
                if current_index == 0 {
                    total_hosts - 1 // Wrap around to the end
                } else {
                    current_index - 1
                }
            }
        };

        log::debug!("Updating selection to index: {}", new_index);

        // Update the index in the ListStateManager
        lstate.select(new_index);

        new_index // Return the computed index
    }).unwrap_or(0) // Default to 0 if mutex lock fails
}

fn filter_entries(entries: &[entry::SshConfigEntry], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..entries.len()).collect();
    }

    let query = query.to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            // Verifica no Host
            entry.host.to_lowercase().contains(&query) ||
            // Verifica apenas no Hostname
            entry.options.iter()
                .find(|(key, value)| key.to_lowercase() == "hostname")
                .map_or(false, |(_, value)| value.to_lowercase().contains(&query))
        })
        .map(|(i, _)| i)
        .collect()
}

fn handle_search_mode(
    event: Event,
    app_mode: &Arc<Mutex<AppMode>>,
    entries: &[entry::SshConfigEntry],
) -> Option<UIEvent> {
    match event {
        Event::Key(key_event) => match key_event.code {
            KeyCode::Esc => {
                log::debug!("ESC pressionado - saindo do modo de busca");
                with_mutex(app_mode, Some("app_mode"), |mode: &mut AppMode| {
                    *mode = AppMode::Normal;
                });
                Some(UIEvent::Normal)
            },
            KeyCode::Char(c) => {
                // Ignorar apenas a tecla '/' quando já estiver no modo de busca
                if c == '/' {
                    return None;
                }
                
                log::debug!("Tecla pressionada: {}", c);
                with_mutex(app_mode, Some("app_mode"), |mode: &mut AppMode| {
                    if let AppMode::Search { query, .. } = mode {
                        let mut new_query = query.clone();
                        new_query.push(c);
                        log::debug!("Nova query: {}", new_query);
                        let matches = filter_entries(entries, &new_query);
                        log::debug!("Encontrados {} matches", matches.len());
                        mode.update_search(new_query, matches);
                    }
                });
                None
            },
            KeyCode::Backspace => {
                log::debug!("Backspace pressionado");
                with_mutex(app_mode, Some("app_mode"), |mode: &mut AppMode| {
                    if let AppMode::Search { query, .. } = mode {
                        let mut new_query = query.clone();
                        new_query.pop();
                        log::debug!("Query após backspace: {}", new_query);
                        let matches = filter_entries(entries, &new_query);
                        log::debug!("Encontrados {} matches", matches.len());
                        mode.update_search(new_query, matches);
                    }
                });
                None
            },
            _ => None,
        },
        _ => None,
    }
}
