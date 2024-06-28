//! This module has functions to handle the reading and writing of the configuration file

use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use crate::entry::SshConfigEntry;


/// Reads and parses the SSH config file at the given path.
///
/// # Arguments
/// * `path` - The path to the SSH config file.
///
/// # Returns
/// A vector of `SshConfigEntry` structs parsed from the file.
pub fn read_ssh_config(path: &str) -> io::Result<Vec<SshConfigEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_host = None;
    let mut options = Vec::new();
    let mut comments = Vec::new();
    let mut current_tag: Option<String> = None;
    let mut previous_tag: Option<String> = None;


    let re_section = Regex::new(r"^\s*Host\s+(.+?)\s*$").unwrap();
    let re_option = Regex::new(r"^\s*(\S+)\s+(.+?)\s*$").unwrap();
    let re_comment = Regex::new(r"^\s*#.*$").unwrap();  // Catch all comment lines
    let re_tag = Regex::new(r"^\s*# -+ ([^-\n]+) -+\s*$").unwrap();  // Specific tag format

    for line in reader.lines() {
        let line = line?.trim().to_string();

        if let Some(caps) = re_tag.captures(&line) {
            // Update the current tag when a tag-like comment is found
            current_tag = Some(caps[1].trim().to_string());
            // Continue to the next iteration to prevent adding tags as comments
            continue;
        }
        if re_comment.is_match(&line) {
            // Before adding the comment, check if it's not a tag line
            if re_tag.is_match(&line) {
                continue; // Skip this as it's a tag, not a regular comment
            } else {
                comments.push(line);
            }
        } else if let Some(caps) = re_section.captures(&line) {
            if let Some(host) = current_host.replace(caps[1].to_string()) {
                entries.push(SshConfigEntry {
                    host,
                    options: options.drain(..).collect(),
                    comments: comments.drain(..).collect(),
                    tag: previous_tag.clone(),
                });
            }
        } else if let Some(caps) = re_option.captures(&line) {
            options.push((caps[1].to_string(), caps[2].to_string()));
        }

        //println!("current = {}, previous = {}",
        //         current_tag.as_ref().map_or("None", String::as_str),
        //         previous_tag.as_ref().map_or("None", String::as_str));

        if current_tag != previous_tag {
            previous_tag = current_tag.clone();
        }
    }

    if let Some(host) = current_host {
        entries.push(SshConfigEntry {
            host,
            options,
            comments,
            tag: previous_tag
        });
    }

    Ok(entries)
}
