use crate::ui::theme::Theme;
use crate::ui::UiState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_help_modal(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(
            " ❓ Ototunetake Keybindings & Help ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_lines = vec![
        Line::from(vec![Span::styled(
            "Playback Controls:",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Space        ", Style::default().fg(Theme::PRIMARY)),
            Span::styled(
                "Play / Pause reference audio & shadowing loop",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  n / Right    ", Style::default().fg(Theme::PRIMARY)),
            Span::styled("Next sentence", Style::default().fg(Theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("  p / Left     ", Style::default().fg(Theme::PRIMARY)),
            Span::styled("Previous sentence", Style::default().fg(Theme::TEXT)),
        ]),
        Line::from(vec![
            Span::styled("  r            ", Style::default().fg(Theme::PRIMARY)),
            Span::styled(
                "Replay current reference audio",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  m / Tab      ", Style::default().fg(Theme::PRIMARY)),
            Span::styled(
                "Cycle preset: [Listen] -> [Repeat] -> [Shadow]",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  s            ", Style::default().fg(Theme::PRIMARY)),
            Span::styled(
                "Toggle Shuffle / Sequential playback",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  + / -        ", Style::default().fg(Theme::PRIMARY)),
            Span::styled("Adjust volume", Style::default().fg(Theme::TEXT)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Display & Learning Options:",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  t            ", Style::default().fg(Theme::ACCENT)),
            Span::styled(
                "Toggle Japanese text (Audio-First Mode)",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  f            ", Style::default().fg(Theme::ACCENT)),
            Span::styled(
                "Toggle Furigana reading display",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  e            ", Style::default().fg(Theme::ACCENT)),
            Span::styled(
                "Toggle English translation display",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  [ / ]        ", Style::default().fg(Theme::ACCENT)),
            Span::styled(
                "Adjust shadowing gap duration (-0.5s / +0.5s)",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  1 - 5        ", Style::default().fg(Theme::ACCENT)),
            Span::styled(
                "Set repeat count (1 to 5 repetitions)",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation & Modals:",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  P            ", Style::default().fg(Theme::WARNING)),
            Span::styled(
                "Open Playlists modal (Mature, Young, Decks, Tags)",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  /            ", Style::default().fg(Theme::WARNING)),
            Span::styled(
                "Instant live search / sentence filter",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Esc / ?      ", Style::default().fg(Theme::WARNING)),
            Span::styled(
                "Close modal / Toggle this help screen",
                Style::default().fg(Theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+C   ", Style::default().fg(Theme::MUTED)),
            Span::styled("Save state and quit", Style::default().fg(Theme::TEXT)),
        ]),
    ];

    let p = Paragraph::new(help_lines).wrap(Wrap { trim: true });
    frame.render_widget(p, inner);
}

pub fn render_playlist_modal(frame: &mut Frame, ui_state: &UiState) {
    let area = centered_rect(50, 45, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT))
        .title(Span::styled(
            " 📑 Select Playlist (↑/↓ to navigate, Enter to select, Esc to close) ",
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = ui_state
        .available_playlists
        .iter()
        .enumerate()
        .map(|(idx, pl)| {
            let is_selected = idx == ui_state.playlist_cursor;
            let marker = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Theme::PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::TEXT)
            };
            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(pl.name(), style),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

pub fn render_search_modal(frame: &mut Frame, ui_state: &UiState) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::WARNING))
        .title(Span::styled(
            " 🔍 Search Sentences (Type query, Enter to filter, Esc to cancel) ",
            Style::default()
                .fg(Theme::WARNING)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = format!("Query: {}█", ui_state.search_query);
    let p = Paragraph::new(text)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Theme::TEXT));
    frame.render_widget(p, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
