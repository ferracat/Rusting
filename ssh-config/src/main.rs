mod config;  // This line tells Rust to include the `config.rs` file as a module

fn main() {
    let home_dir = shellexpand::tilde("~").into_owned();
    let config_path = format!("{}/.ssh/config", home_dir);
    let configs = config::read_ssh_config(&config_path);

    match configs {
        Ok(entries) => {
            for entry in entries {
                println!("\nHost: {}", entry.host);
                // If you want to print options as well:
                for (key, value) in entry.options {
                    println!("  {} => {}", key, value);
                }
                for (comment) in entry.comments {
                    println!("  // {}", comment);
                }
                let tag_display = entry.tag.as_ref().map_or("None", String::as_str);
                println!("  Tag: {}", tag_display);            }
        },
        Err(e) => {
            eprintln!("Failed to read SSH config: {}", e);
        }
    }
}
