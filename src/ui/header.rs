use crate::domain::config::PlaybackPreset;
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

    frame.render_widget(Paragraph::new(build_header_top_line(player)), chunks[0]);
    frame.render_widget(
        Paragraph::new(build_header_bottom_line(queue, stats)),
        chunks[1],
    );
}

fn build_header_top_line(player: &ShadowPlayer) -> Line<'static> {
    let status_badge = build_status_badge(player.status(), player.config().preset);
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

    let gap_val = match player.config().preset {
        PlaybackPreset::Shadow => player.config().shadow_breath_gap_secs,
        PlaybackPreset::Echo => player.effective_echo_gap().as_secs_f32(),
        PlaybackPreset::Listen => player.config().listen_gap_secs,
    };

    Line::from(vec![
        Span::raw("Status: "),
        status_badge,
        Span::raw("  "),
        preset_badge,
        Span::raw("  "),
        rep_span,
        Span::raw("  •  Gap: "),
        Span::styled(
            format!("{gap_val:.1}s"),
            Style::default().fg(Theme::WARNING),
        ),
        Span::raw("  •  Vol: "),
        Span::styled(
            format!("{:.0}%", player.audio().volume() * 100.0),
            Style::default().fg(Theme::SECONDARY),
        ),
    ])
}

fn build_header_bottom_line(queue: &SentenceQueue, stats: &PracticeStats) -> Line<'static> {
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

    Line::from(vec![
        Span::styled(queue_info, Style::default().fg(Theme::TEXT)),
        Span::raw("  │  "),
        Span::styled(stats_info, Style::default().fg(Theme::MUTED)),
    ])
}

fn build_status_badge(status: PlaybackStatus, preset: PlaybackPreset) -> Span<'static> {
    match status {
        PlaybackStatus::Playing => badge_playing(preset),
        PlaybackStatus::Replaying => badge_replaying(),
        PlaybackStatus::SpeakingGap => badge_speaking_gap(),
        PlaybackStatus::BreathingGap => badge_breathing_gap(),
        PlaybackStatus::CardComplete => badge_card_complete(),
        PlaybackStatus::Paused => badge_paused(),
        PlaybackStatus::Stopped => badge_stopped(),
    }
}

fn badge_playing(preset: PlaybackPreset) -> Span<'static> {
    if preset == PlaybackPreset::Shadow {
        Span::styled(
            " 🗣 SHADOWING ",
            Style::default()
                .bg(Theme::SUCCESS)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " ▶ LISTENING ",
            Style::default()
                .bg(Theme::SUCCESS)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
    }
}

fn badge_replaying() -> Span<'static> {
    Span::styled(
        " ▶ REPLAY CHECK ",
        Style::default()
            .bg(Theme::SECONDARY)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}

fn badge_speaking_gap() -> Span<'static> {
    Span::styled(
        " 🗣 ECHO (REPEAT) ",
        Style::default()
            .bg(Theme::ACCENT)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}

fn badge_breathing_gap() -> Span<'static> {
    Span::styled(
        " ⏸ BREATH GAP ",
        Style::default()
            .bg(Theme::MUTED)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}

fn badge_card_complete() -> Span<'static> {
    Span::styled(
        " 🎉 CARD COMPLETE ",
        Style::default()
            .bg(Theme::WARNING)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}

fn badge_paused() -> Span<'static> {
    Span::styled(
        " ⏸ PAUSED ",
        Style::default()
            .bg(Theme::WARNING)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}

fn badge_stopped() -> Span<'static> {
    Span::styled(
        " ⏹ STOPPED ",
        Style::default().bg(Theme::MUTED).fg(Color::Black),
    )
}
