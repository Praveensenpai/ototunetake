use crate::domain::playlist::PlaylistFilter;
use crate::engine::player::ShadowPlayer;
use crate::engine::queue::SentenceQueue;
use crate::error::Result;
use crate::ui::UiState;
use crossterm::event::{self, KeyCode, KeyModifiers};

pub fn handle_key_event(
    key: event::KeyEvent,
    player: &mut ShadowPlayer,
    queue: &mut SentenceQueue,
    ui_state: &mut UiState,
) -> Result<bool> {
    if ui_state.show_search {
        return handle_search_key(key, queue, ui_state);
    }
    if ui_state.show_playlists {
        return handle_playlist_key(key, queue, ui_state);
    }
    if ui_state.show_help {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
        ) {
            ui_state.show_help = false;
        }
        return Ok(false);
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) | (_, KeyCode::Char('q')) => Ok(true),
        (_, KeyCode::Char(' ')) => {
            if let Err(e) = player.toggle_play(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(false)
        }
        (_, KeyCode::Char('n')) | (_, KeyCode::Right) => {
            if let Err(e) = player.next(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(false)
        }
        (_, KeyCode::Char('p')) | (_, KeyCode::Left) => {
            if let Err(e) = player.previous(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(false)
        }
        (_, KeyCode::Char('r')) => {
            if let Err(e) = player.replay(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(false)
        }
        (_, KeyCode::Char('m')) | (_, KeyCode::Tab) => {
            let next_preset = player.config().preset.cycle();
            player.config_mut().preset = next_preset;
            ui_state.set_status(format!("Preset switched to [{}]", next_preset.name()));
            Ok(false)
        }
        (_, KeyCode::Char('s')) => {
            queue.toggle_shuffle();
            let state_str = if queue.is_shuffled() { "ON" } else { "OFF" };
            ui_state.set_status(format!("Shuffle: {state_str}"));
            Ok(false)
        }
        (_, KeyCode::Char('t')) => {
            player.config_mut().show_japanese = !player.config().show_japanese;
            let status = if player.config().show_japanese {
                "Visible"
            } else {
                "Hidden (Audio-First)"
            };
            ui_state.set_status(format!("Japanese text: {status}"));
            Ok(false)
        }
        (_, KeyCode::Char('f')) => {
            player.config_mut().show_furigana = !player.config().show_furigana;
            Ok(false)
        }
        (_, KeyCode::Char('e')) => {
            player.config_mut().show_translation = !player.config().show_translation;
            Ok(false)
        }
        (_, KeyCode::Char('+')) | (_, KeyCode::Char('=')) => {
            let cur = player.audio().volume();
            player.audio_mut().set_volume(cur + 0.05);
            player.config_mut().volume = player.audio().volume();
            ui_state.set_status(format!("Volume: {:.0}%", player.audio().volume() * 100.0));
            Ok(false)
        }
        (_, KeyCode::Char('-')) => {
            let cur = player.audio().volume();
            player.audio_mut().set_volume(cur - 0.05);
            player.config_mut().volume = player.audio().volume();
            ui_state.set_status(format!("Volume: {:.0}%", player.audio().volume() * 100.0));
            Ok(false)
        }
        (_, KeyCode::Char('[')) => {
            let cur = player.config().shadow_gap_secs;
            player.config_mut().shadow_gap_secs = (cur - 0.5).max(0.0);
            ui_state.set_status(format!(
                "Shadow gap: {:.1}s",
                player.config().shadow_gap_secs
            ));
            Ok(false)
        }
        (_, KeyCode::Char(']')) => {
            let cur = player.config().shadow_gap_secs;
            player.config_mut().shadow_gap_secs = (cur + 0.5).min(15.0);
            ui_state.set_status(format!(
                "Shadow gap: {:.1}s",
                player.config().shadow_gap_secs
            ));
            Ok(false)
        }
        (_, KeyCode::Char(c)) if ('1'..='5').contains(&c) => {
            let count = c.to_digit(10).unwrap_or(2);
            player.config_mut().repeat_count = count;
            ui_state.set_status(format!("Repetition count: {count}"));
            Ok(false)
        }
        (_, KeyCode::Char('P')) => {
            ui_state.show_playlists = true;
            Ok(false)
        }
        (_, KeyCode::Char('/')) => {
            ui_state.show_search = true;
            ui_state.search_query.clear();
            Ok(false)
        }
        (_, KeyCode::Char('?')) => {
            ui_state.show_help = true;
            Ok(false)
        }
        _ => Ok(false),
    }
}

pub fn handle_search_key(
    key: event::KeyEvent,
    queue: &mut SentenceQueue,
    ui_state: &mut UiState,
) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            ui_state.show_search = false;
        }
        KeyCode::Enter => {
            let query = ui_state.search_query.trim().to_string();
            ui_state.show_search = false;
            if query.is_empty() {
                queue.set_filter(PlaylistFilter::AllReviewed);
                ui_state.set_status("Filter cleared: All Reviewed");
            } else {
                queue.set_filter(PlaylistFilter::Search(query.clone()));
                ui_state.set_status(format!(
                    "Filter applied: Search '{query}' ({} results)",
                    queue.total()
                ));
            }
        }
        KeyCode::Backspace => {
            ui_state.search_query.pop();
        }
        KeyCode::Char(c) => {
            ui_state.search_query.push(c);
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_playlist_key(
    key: event::KeyEvent,
    queue: &mut SentenceQueue,
    ui_state: &mut UiState,
) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            ui_state.show_playlists = false;
        }
        KeyCode::Up => {
            if ui_state.playlist_cursor > 0 {
                ui_state.playlist_cursor -= 1;
            } else {
                ui_state.playlist_cursor = ui_state.available_playlists.len().saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if ui_state.playlist_cursor + 1 < ui_state.available_playlists.len() {
                ui_state.playlist_cursor += 1;
            } else {
                ui_state.playlist_cursor = 0;
            }
        }
        KeyCode::Enter => {
            if let Some(selected) = ui_state
                .available_playlists
                .get(ui_state.playlist_cursor)
                .cloned()
            {
                queue.set_filter(selected.clone());
                ui_state.set_status(format!(
                    "Loaded playlist: {} ({} sentences)",
                    selected.name(),
                    queue.total()
                ));
            }
            ui_state.show_playlists = false;
        }
        _ => {}
    }
    Ok(false)
}
