use crate::domain::stats::PracticeStats;
use crate::engine::player::{PlaybackStatus, ShadowPlayer};
use crate::engine::queue::SentenceQueue;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_header(
    frame: &mut Frame,
    player: &ShadowPlayer,
    queue: &SentenceQueue,
    stats: &PracticeStats,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(
            " 🌸 Ototunetake ── Japanese Shadowing Player ",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let status_badge = match player.status() {
        PlaybackStatus::Playing => Span::styled(
            " ▶ PLAYING ",
            Style::default()
                .bg(Theme::SUCCESS)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        PlaybackStatus::Replaying => Span::styled(
            " ▶ REPLAY CHECK ",
            Style::default()
                .bg(Theme::SECONDARY)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        PlaybackStatus::Paused => Span::styled(
            " ⏸ PAUSED ",
            Style::default()
                .bg(Theme::WARNING)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        PlaybackStatus::WaitingGap => Span::styled(
            " 🗣 SHADOW GAP ",
            Style::default()
                .bg(Theme::ACCENT)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        PlaybackStatus::Stopped => Span::styled(
            " ⏹ STOPPED ",
            Style::default().bg(Theme::MUTED).fg(Color::Black),
        ),
    };

    let preset_badge = Span::styled(
        format!(" [{}] ", player.config().preset.name().to_uppercase()),
        Style::default()
            .fg(Theme::SECONDARY)
            .add_modifier(Modifier::BOLD),
    );

    let rep_str = format!(
        "Rep {}/{}",
        player.current_repetition(),
        player.config().repeat_count
    );
    let rep_span = Span::styled(rep_str, Style::default().fg(Theme::TEXT));

    let line1 = Line::from(vec![
        Span::raw("Status: "),
        status_badge,
        Span::raw("  "),
        preset_badge,
        Span::raw("  "),
        rep_span,
        Span::raw("  •  Gap: "),
        Span::styled(
            format!("{:.1}s", player.config().shadow_gap_secs),
            Style::default().fg(Theme::WARNING),
        ),
        Span::raw("  •  Vol: "),
        Span::styled(
            format!("{:.0}%", player.audio().volume() * 100.0),
            Style::default().fg(Theme::SECONDARY),
        ),
    ]);

    let queue_info = if queue.is_empty() {
        "Queue empty".to_string()
    } else {
        format!(
            "Sentence {} of {} ({})",
            queue.cursor() + 1,
            queue.total(),
            queue.active_filter().name()
        )
    };

    let mins = (stats.today_listening_secs / 60.0).round() as u64;
    let stats_info = format!(
        "Today: {} sent ({}m, {} reps)",
        stats.today_sentences_count, mins, stats.today_repetitions_count
    );

    let line2 = Line::from(vec![
        Span::styled(queue_info, Style::default().fg(Theme::TEXT)),
        Span::raw("  │  "),
        Span::styled(stats_info, Style::default().fg(Theme::MUTED)),
    ]);

    frame.render_widget(Paragraph::new(line1), chunks[0]);
    frame.render_widget(Paragraph::new(line2), chunks[1]);
}
