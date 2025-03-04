use crate::entry::SshConfigEntry;

use ratatui as tui;
use tui::{
    layout,
    style::{Color, Modifier, Style},
    text::{Span, Text, Line},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame,
};

use crate::app::AppMode;

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

pub fn render_search_bar(frame: &mut Frame, area: layout::Rect, app_mode: &AppMode) {
    if let AppMode::Search { query, matches, .. } = app_mode {
        let style = Style::default().fg(Color::Yellow);
        
        // O título agora mostra o número de matches
        let title = if matches.is_empty() && query.is_empty() {
            " Search ".to_string()
        } else if matches.is_empty() {
            " Search (no matches) ".to_string()
        } else {
            format!(" Search ({} matches) ", matches.len())
        };

        // O conteúdo mostra apenas o texto digitado ou a mensagem inicial
        let content = if query.is_empty() {
            "type to search...".to_string()
        } else {
            query.clone()
        };

        let paragraph = Paragraph::new(content)
            .style(style)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(title)
                .title_style(style));

        frame.render_widget(paragraph, area);
    }
}

pub fn highlight_search_matches(text: &str, query: &str) -> Text<'static> {
    if query.is_empty() {
        return Text::from(text.to_string());
    }

    let mut spans = Vec::new();
    let query = query.to_lowercase();
    let mut last_end = 0;

    for (start, _) in text.to_lowercase().match_indices(&query) {
        // Add the text before the match
        if start > last_end {
            spans.push(Span::raw(text[last_end..start].to_string()));
        }

        // Add the highlighted match
        spans.push(Span::styled(
            text[start..start + query.len()].to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

        last_end = start + query.len();
    }

    // Add any remaining text
    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    Text::from(vec![Line::from(spans)])
}
