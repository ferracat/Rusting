use crate::entry::SshConfigEntry;

use ratatui as tui;
use tui::{
    layout,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};


pub fn render_popup_table(f: &mut Frame, area: layout::Rect, entry: &SshConfigEntry) {
    let popup_block = Block::default()
        .title(Span::styled(
            format!(" {} ", entry.host),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Black));

    // Collect rows for each field in the entry
    let mut rows = vec![
        Row::new(vec![Cell::from("Host"), Cell::from(entry.host.clone())]),
    ];

    // Add each option as a row
    for (key, value) in &entry.options {
        rows.push(Row::new(vec![Cell::from(key.clone()), Cell::from(value.clone())]));
    }

    // Add each comment as a row
    for comment in &entry.comments {
        rows.push(Row::new(vec![Cell::from("Comment"), Cell::from(comment.clone())]));
    }

    // Add tag if it exists
    if let Some(tag) = &entry.tag {
        rows.push(Row::new(vec![Cell::from("Tag"), Cell::from(tag.clone())]));
    }

    let table = Table::new(
        rows,
        &[
            layout::Constraint::Percentage(40),
            layout::Constraint::Percentage(60),
        ],
    )
    .block(popup_block)
    .style(Style::default().fg(Color::White));

    f.render_widget(table, area);
}
