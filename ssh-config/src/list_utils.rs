// src/list_utils.rs

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ListStateManager {
    pub scroll_offset: usize,
    selected_index: usize,
    pub max_display_items: usize,
    state: ListState,
}

impl ListStateManager {
    /// Create a new ListStateManager with an initial scroll offset and max display items
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            selected_index: 0,
            max_display_items: 0,
            state: ListState::default(),
        }
    }

    /// Update the selected index and adjust scroll offset if needed
    pub fn select(&mut self, index: usize) {
        self.state.select(Some(index));
        self.selected_index = index;

        // Ensure the selected item is within the visible range
        if index < self.scroll_offset {
            self.scroll_offset = index;
        } else if index >= self.scroll_offset + self.max_display_items {
            self.scroll_offset = index - self.max_display_items;
        }
    }

    /// Retrieves the currently selected index
    pub fn get_index(&self) -> usize {
        self.selected_index
    }

    // Get a mutable reference to `list_state` for rendering.
    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.state
    }
}
