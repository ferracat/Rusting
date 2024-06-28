mod config;
mod entry;  // This line tells Rust to include the `config.rs` file as a module

use crossterm;
use tui;

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
    crossterm::terminal::enable_raw_mode();

    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    // Create the list of hostnames
    let hosts: Vec<tui::widgets::ListItem> = entries
        .iter()
        .map(|entry| {
            tui::widgets::ListItem::new(tui::text::Spans::from(tui::text::Span::raw(&entry.host)))
                .style(tui::style::Style::default().fg(tui::style::Color::White))
        })
        .collect();

    // Main loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = tui::layout::Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .margin(1)
                .constraints([tui::layout::Constraint::Percentage(100)].as_ref())
                .split(size);

            let list = tui::widgets::List::new(hosts.clone())
                .block(tui::widgets::Block::default().borders(tui::widgets::Borders::ALL).title(" SSH Hosts "))
                .highlight_style(tui::style::Style::default().fg(tui::style::Color::Yellow))
                .highlight_symbol(">> ");

            f.render_widget(list, chunks[0]);
        })?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.code == crossterm::event::KeyCode::Char('q') {
                break;
            }
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
