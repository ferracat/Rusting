// src/list_utils.rs

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ListStateManager {
    pub scroll_offset: usize,
    selected_index: usize,
    pub max_display_items: Option<usize>,
    state: ListState,
}

impl ListStateManager {
    /// Create a new ListStateManager with an initial scroll offset and max display items
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            selected_index: 0,
            max_display_items: None,
            state: ListState::default(),
        }
    }

    /// Update the selected index and adjust scroll offset if needed
    pub fn select(&mut self, index: usize, total_items: usize) {
        self.state.select(Some(index));
        self.selected_index = index;

        // Ensure the selected item is within the visible range
        if index < self.scroll_offset {
            self.scroll_offset = index;
        } else if index >= self.scroll_offset + self.max_display_items.unwrap_or(0) {
            self.scroll_offset = index + 1 - self.max_display_items.unwrap_or(0);
        }

        // Clamp the scroll offset to prevent overflow
        self.scroll_offset = self.scroll_offset.min(total_items.saturating_sub(self.max_display_items.unwrap_or(0)));
    }

    /// Retrieves the currently selected index
    pub fn get_index(&self) -> usize {
        self.selected_index
    }

    /// Sets the scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Clear the current selection.
    pub fn clear_selection(&mut self) {
        self.state.select(None);
    }

    /// Scroll up by one item
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down by one item, keeping within bounds of total items
    pub fn scroll_down(&mut self, total_items: usize) {
        if self.scroll_offset + self.max_display_items.unwrap_or(0) < total_items {
            self.scroll_offset += 1;
        }
    }

    // Get a mutable reference to `list_state` for rendering.
    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.state
    }
}
