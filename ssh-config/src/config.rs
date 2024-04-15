use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
pub struct SshConfigEntry {
    pub host: String,
    pub options: Vec<(String, String)>,
    pub comments: Vec<String>,
    pub tag: Option<String>,
}

pub fn read_ssh_config(path: &str) -> io::Result<Vec<SshConfigEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_host = None;
    let mut options = Vec::new();
    let mut comments = Vec::new();
    let mut current_tag = None;

    let re_section = Regex::new(r"^\s*Host\s+(.+?)\s*$").unwrap();
    let re_option = Regex::new(r"^\s*(\S+)\s+(.+?)\s*$").unwrap();
    let re_comment = Regex::new(r"^\s*#([^-\n\r]*)").unwrap();
    let re_tag = Regex::new(r"^\s*# --- ([^---]+) ---\s*$").unwrap();

    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re_section.captures(&line) {
            if let Some(host) = current_host.replace(caps[1].to_string()) {
                entries.push(SshConfigEntry {
                    host,
                    options: options.drain(..).collect(),
                    comments: comments.drain(..).collect(),
                    tag: current_tag.clone(),
                });
            }
        } else if let Some(caps) = re_option.captures(&line) {
            options.push((caps[1].to_string(), caps[2].to_string()));
        } else if let Some(caps) = re_comment.captures(&line) {
            comments.push(caps[1].trim().to_string());
        }
    }

    if let Some(host) = current_host {
        entries.push(SshConfigEntry {
            host,
            options,
            comments,
            tag: current_tag
        });
    }

    Ok(entries)
}
