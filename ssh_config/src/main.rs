mod config;

fn main() {
    match config::read_ssh_config() {
        Ok(ssh_config) => {
            println!("{:#?}", ssh_config);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Example of writing SSH config
    // let ssh_config = SSHConfig::new();
    // match config::write_ssh_config(&ssh_config) {
    //     Ok(_) => println!("SSH config written successfully"),
    //     Err(e) => eprintln!("Error writing SSH config: {}", e),
    // }
}
