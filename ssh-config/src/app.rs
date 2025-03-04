//! This module has is used for the ratatui App

// Define the AppMode enum here
#[derive(Debug)]
pub enum AppMode {
    Normal,
    Help,
    Search {
        query: String,
        cursor_position: usize,
        matches: Vec<usize>,
        current_match: Option<usize>,
    },
}

impl AppMode {
    pub fn is_search(&self) -> bool {
        matches!(self, AppMode::Search { .. })
    }

    pub fn get_search_query(&self) -> Option<&str> {
        match self {
            AppMode::Search { query, .. } => Some(query),
            _ => None,
        }
    }

    pub fn update_search(&mut self, new_query: String, matches: Vec<usize>) {
        if let AppMode::Search { query, cursor_position, .. } = self {
            *query = new_query;
            *cursor_position = query.len();
            *self = AppMode::Search {
                query: query.clone(),
                cursor_position: *cursor_position,
                matches,
                current_match: None,
            };
        }
    }
}

