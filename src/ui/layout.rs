use crate::ui::theme::Theme;
use crate::ui::UiState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn create_main_layout(area: Rect) -> Vec<Rect> {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(12),   // Body
            Constraint::Length(2), // Footer
        ])
        .split(area);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Player Card
            Constraint::Percentage(40), // Queue View
        ])
        .split(vertical_chunks[1]);

    vec![
        vertical_chunks[0],
        body_chunks[0],
        body_chunks[1],
        vertical_chunks[2],
    ]
}

pub fn render_footer(frame: &mut Frame, ui_state: &UiState, area: Rect) {
    let content = if let Some(status) = ui_state.current_status() {
        vec![
            Span::styled(" 🔔 ", Style::default().fg(Theme::WARNING)),
            Span::styled(status, Style::default().fg(Theme::TEXT)),
        ]
    } else {
        vec![
            Span::styled(" [Space]", Style::default().fg(Theme::PRIMARY)),
            Span::styled(" Play/Pause ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [n/p]", Style::default().fg(Theme::PRIMARY)),
            Span::styled(" Next/Prev ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [r]", Style::default().fg(Theme::PRIMARY)),
            Span::styled(" Replay ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [m]", Style::default().fg(Theme::SECONDARY)),
            Span::styled(" Mode ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [t]", Style::default().fg(Theme::SECONDARY)),
            Span::styled(" Text ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [s]", Style::default().fg(Theme::SECONDARY)),
            Span::styled(" Shuffle ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [P]", Style::default().fg(Theme::ACCENT)),
            Span::styled(" Playlist ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [/]", Style::default().fg(Theme::ACCENT)),
            Span::styled(" Search ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [?]", Style::default().fg(Theme::WARNING)),
            Span::styled(" Help ", Style::default().fg(Theme::MUTED)),
            Span::styled(" [q]", Style::default().fg(Theme::MUTED)),
            Span::styled(" Quit", Style::default().fg(Theme::MUTED)),
        ]
    };

    let paragraph = Paragraph::new(Line::from(content));
    frame.render_widget(paragraph, area);
}
