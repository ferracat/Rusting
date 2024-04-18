
#[derive(Debug)]
pub struct SshConfigEntry {
    pub host: String,
    pub options: Vec<(String, String)>,
    pub comments: Vec<String>,
    pub tag: Option<String>, // it is option so that it can be None or String.
}

impl SshConfigEntry {

    pub fn set_tag(&mut self, tag: String) {
        self.tag = Some(tag);
    }
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
