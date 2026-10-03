use crate::domain::config::AppConfig;
use crate::engine::player::ShadowPlayer;
use crate::engine::queue::SentenceQueue;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

pub fn render_player_card(
    frame: &mut Frame,
    player: &ShadowPlayer,
    queue: &SentenceQueue,
    config: &AppConfig,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::SECONDARY))
        .title(Span::styled(
            " 🎧 Japanese Shadowing Player ",
            Style::default()
                .fg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(sentence) = queue.current() else {
        let empty_p = Paragraph::new("No sentence currently loaded. Check your playlist filter.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Theme::MUTED));
        frame.render_widget(empty_p, inner);
        return;
    };

    let card_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top spacing
            Constraint::Min(3),    // Japanese sentence text & translation
            Constraint::Length(1), // Audio progress or Gap gauge
            Constraint::Length(1), // Spacing
            Constraint::Length(2), // Active loop settings (Mode, Reps, Gap, Auto-Next)
            Constraint::Length(2), // Card metadata (deck, tags, reps, interval)
        ])
        .split(inner);

    render_sentence_text(frame, sentence, config, card_chunks[1]);
    render_progress_bar(frame, player, card_chunks[2]);
    render_controls_helper(frame, config, player.effective_shadow_gap(), card_chunks[4]);
    render_metadata(frame, sentence, card_chunks[5]);
}

fn render_sentence_text(
    frame: &mut Frame,
    sentence: &crate::domain::sentence::Sentence,
    config: &AppConfig,
    area: Rect,
) {
    let mut lines = Vec::new();

    if config.show_japanese {
        lines.push(Line::from(vec![Span::styled(
            &sentence.japanese_text,
            Style::default()
                .fg(Theme::TEXT)
                .add_modifier(Modifier::BOLD),
        )]));

        if config.show_furigana {
            if let Some(furi) = &sentence.furigana {
                lines.push(Line::from(vec![Span::styled(
                    furi,
                    Style::default().fg(Theme::MUTED),
                )]));
            }
        }

        if config.show_translation {
            if let Some(trans) = &sentence.translation {
                lines.push(Line::from(vec![Span::styled(
                    trans,
                    Style::default().fg(Theme::SECONDARY),
                )]));
            }
        }
    } else {
        lines.push(Line::from(vec![Span::styled(
            "🔊  [ Japanese Text Hidden — Audio-First Listening ]",
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "Press 't' to toggle sentence text, 'f' for furigana, 'e' for translation",
            Style::default().fg(Theme::MUTED),
        )]));
    }

    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_progress_bar(frame: &mut Frame, player: &ShadowPlayer, area: Rect) {
    match player.phase() {
        crate::engine::player::ShadowLoopPhase::ShadowGap {
            start, duration, ..
        } => {
            let elapsed = start.elapsed().min(*duration);
            let ratio = if duration.as_secs_f32() > 0.0 {
                (elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let label = format!(
                "🗣 Your Turn (Shadow): {:.1}s / {:.1}s",
                elapsed.as_secs_f32(),
                duration.as_secs_f32()
            );
            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Theme::ACCENT)
                        .bg(Color::Rgb(60, 60, 70)),
                )
                .ratio(ratio as f64)
                .label(label);
            frame.render_widget(gauge, area);
        }
        crate::engine::player::ShadowLoopPhase::PostReplayGap {
            start, duration, ..
        } => {
            let elapsed = start.elapsed().min(*duration);
            let ratio = if duration.as_secs_f32() > 0.0 {
                (elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let label = format!(
                "⏸ Next In: {:.1}s / {:.1}s",
                elapsed.as_secs_f32(),
                duration.as_secs_f32()
            );
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Theme::MUTED).bg(Color::Rgb(60, 60, 70)))
                .ratio(ratio as f64)
                .label(label);
            frame.render_widget(gauge, area);
        }
        crate::engine::player::ShadowLoopPhase::PlayingReference { .. } => {
            let elapsed = player.audio().elapsed().as_secs_f32();
            let total = player.audio().duration().as_secs_f32();
            let ratio = player.audio().progress_ratio();
            let label = format!("▶ Reference Audio: {:.1}s / {:.1}s", elapsed, total);
            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Theme::PRIMARY)
                        .bg(Color::Rgb(60, 60, 70)),
                )
                .ratio(ratio)
                .label(label);
            frame.render_widget(gauge, area);
        }
        crate::engine::player::ShadowLoopPhase::PlayingReplay { .. } => {
            let elapsed = player.audio().elapsed().as_secs_f32();
            let total = player.audio().duration().as_secs_f32();
            let ratio = player.audio().progress_ratio();
            let label = format!("▶ Replay Check: {:.1}s / {:.1}s", elapsed, total);
            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Theme::SECONDARY)
                        .bg(Color::Rgb(60, 60, 70)),
                )
                .ratio(ratio)
                .label(label);
            frame.render_widget(gauge, area);
        }
        crate::engine::player::ShadowLoopPhase::Paused { .. } => {
            let elapsed = player.audio().elapsed().as_secs_f32();
            let total = player.audio().duration().as_secs_f32();
            let ratio = player.audio().progress_ratio();
            let label = format!("⏸ Paused at: {:.1}s / {:.1}s", elapsed, total);
            let gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Theme::WARNING)
                        .bg(Color::Rgb(60, 60, 70)),
                )
                .ratio(ratio)
                .label(label);
            frame.render_widget(gauge, area);
        }
        crate::engine::player::ShadowLoopPhase::Stopped => {
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Theme::MUTED).bg(Color::Rgb(60, 60, 70)))
                .percent(0)
                .label("Audio Ready — Press [Space] to Play");
            frame.render_widget(gauge, area);
        }
    }
}

fn render_metadata(frame: &mut Frame, sentence: &crate::domain::sentence::Sentence, area: Rect) {
    let line1 = Line::from(vec![
        Span::styled("Deck: ", Style::default().fg(Theme::MUTED)),
        Span::styled(&sentence.deck, Style::default().fg(Theme::TEXT)),
        Span::raw("  │  "),
        Span::styled("Tags: ", Style::default().fg(Theme::MUTED)),
        Span::styled(
            sentence.display_tags(),
            Style::default().fg(Theme::SECONDARY),
        ),
    ]);

    let stat_info = format!(
        "Status: {} │ Ivl: {}d │ Reps: {} │ Played: {}x • Shadowed: {}x",
        sentence.status.as_str(),
        sentence.interval,
        sentence.reps,
        sentence.play_count,
        sentence.shadow_count
    );
    let line2 = Line::from(Span::styled(stat_info, Style::default().fg(Theme::MUTED)));

    let p = Paragraph::new(vec![line1, line2])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_controls_helper(
    frame: &mut Frame,
    config: &AppConfig,
    effective_gap: std::time::Duration,
    area: Rect,
) {
    let mode_desc = match config.preset {
        crate::domain::config::PlaybackPreset::Shadow => "Listen → Shadow Gap → Replay Check",
        crate::domain::config::PlaybackPreset::Repeat => "Listen → Silence Gap → Listen",
        crate::domain::config::PlaybackPreset::Listen => "Continuous Native Audio (No Gaps)",
    };

    let line1 = Line::from(vec![
        Span::styled(" Mode [m]: ", Style::default().fg(Theme::SECONDARY)),
        Span::styled(
            format!("{:<7} ", config.preset.name().to_uppercase()),
            Style::default()
                .fg(Theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("({mode_desc})"), Style::default().fg(Theme::MUTED)),
    ]);

    let rep_str = format!("{}x [keys 1-5]", config.repeat_count);
    let gap_str = format!("{:.1}s (auto)", effective_gap.as_secs_f32());
    let adv_str = if config.auto_advance {
        "ON [key a]"
    } else {
        "OFF (loop 1 sentence) [key a]"
    };

    let line2 = Line::from(vec![
        Span::styled(" Reps: ", Style::default().fg(Theme::PRIMARY)),
        Span::styled(rep_str, Style::default().fg(Theme::TEXT)),
        Span::raw("   │   "),
        Span::styled("Gap: ", Style::default().fg(Theme::WARNING)),
        Span::styled(gap_str, Style::default().fg(Theme::TEXT)),
        Span::raw("   │   "),
        Span::styled("Auto-Advance: ", Style::default().fg(Theme::ACCENT)),
        Span::styled(
            adv_str,
            Style::default().fg(if config.auto_advance {
                Theme::SUCCESS
            } else {
                Theme::WARNING
            }),
        ),
    ]);

    let p = Paragraph::new(vec![line1, line2]).alignment(Alignment::Center);
    frame.render_widget(p, area);
}
