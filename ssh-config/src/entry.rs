//! This module contains the `SshConfigEntry` struct and its associated methods.

use std::fmt;

/// Represents an SSH configuration entry.
/// Each entry is composed of:
/// * host
/// * options (Hash Map)
/// * comments (Vector)
/// * tag
#[derive(Debug, Clone)]
pub struct SshConfigEntry {
    pub host: String,
    pub options: Vec<(String, String)>,
    pub comments: Vec<String>,
    pub tag: Option<String>, // it is option so that it can be None or String.
}

/// This Display is considered a trait and works like a file descriptor that calls the string formatter
/// to represent the element so that when the element is called inside the print function it will be
/// displayed like a pretty print.
///
/// `"println!("{}", entry);`
impl fmt::Display for SshConfigEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Host: {}", self.host)?;
        for (key, value) in &self.options {
            writeln!(f, "  {} => {}", key, value)?;
        }
        for comment in &self.comments {
            writeln!(f, "  // {}", comment)?;
        }
        if let Some(tag) = &self.tag {
            writeln!(f, "  >> {}", tag)?;
        }
        Ok(())
    }
}

impl SshConfigEntry {

    pub fn add_option(&mut self, key: String, value: String) {
        self.options.push((key, value));
    }

    pub fn add_comment(&mut self, comment: String) {
        self.comments.push(comment);
    }

    pub fn set_tag(&mut self, tag: String) {
        self.tag = Some(tag);
    }

    /// This function is supposed to work like the Display trait but instead of just putting the entry
    /// the call to the method *display()* needs to be there.
    ///
    /// `println!("{}", entry.display());`
    pub fn display(&self) -> String {
        let mut result = format!("Host: {}\n", self.host);
        for (key, value) in &self.options {
            result.push_str(&format!("  {} => {}\n", key, value));
        }
        for comment in &self.comments {
            result.push_str(&format!("  // {}\n", comment));
        }
        if let Some(tag) = &self.tag {
            result.push_str(&format!("  >> {}\n", tag));
        }
        result
    }
}

#[derive(Debug, Clone)]
pub enum SshOption {
    HostName(String),
    User(String),
    Port(u16),
    IdentityFile(String),
    // TODO: Add the other options
}
