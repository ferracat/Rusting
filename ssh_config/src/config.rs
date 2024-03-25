use dirs;
use lazy_static::lazy_static;
use ssh_config::SSHConfig;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref CONFIG_FILE_PATH: String = {
        let mut path = dirs::home_dir().expect("Unable to determine home directory");
        path.push(".ssh/config");
        path.to_string_lossy().into_owned()
    };
}

pub fn read_ssh_config() -> io::Result<SSHConfig<'static'>> {
    // Read the content of the SSH config file
    let config_content = fs::read_to_string(&*&CONFIG_FILE_PATHATH)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Config file not found"))?;

    // Parse the content into an SSHConfig object
    let ssh_config = SSHConfig::parse_str(&config_content).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Error parsing SSH configuration",
        )
    })?;

    Ok(ssh_config)
}

pub fn write_ssh_config(ssh_config: &[SSHConfig]) -> io::Result<()> {
    let config_content = ssh_config
        .iter()
        .map_while()
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&*CONFIG_FILE_PATH, config_content)
}
