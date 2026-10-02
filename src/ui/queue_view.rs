use crate::engine::queue::SentenceQueue;
use crate::ui::theme::Theme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

pub fn render_queue_view(frame: &mut Frame, queue: &SentenceQueue, area: ratatui::layout::Rect) {
    let shuffle_badge = if queue.is_shuffled() {
        " [SHUFFLED] "
    } else {
        " [SEQUENTIAL] "
    };

    let title = format!(" 📋 Queue {shuffle_badge} ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::ACCENT))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut items = Vec::new();
    let current_idx = queue.cursor();
    let active = queue.active_sentences();

    let start = current_idx.saturating_sub(1);
    let end = (current_idx + 12).min(active.len());

    for (i, sentence) in active.iter().enumerate().take(end).skip(start) {
        let is_current = i == current_idx;

        let marker = if is_current { "▶ " } else { "  " };
        let prefix = format!("{}{:3}. ", marker, i + 1);

        let style = if is_current {
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::TEXT)
        };

        let tag_summary = if !sentence.tags.is_empty() {
            format!(" [{}]", sentence.tags[0])
        } else {
            String::new()
        };

        let line = Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(&sentence.japanese_text, style),
            Span::styled(tag_summary, Style::default().fg(Theme::MUTED)),
        ]);

        items.push(ListItem::new(line));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  (No upcoming sentences)",
            Style::default().fg(Theme::MUTED),
        ))));
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}
