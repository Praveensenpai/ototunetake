use crate::domain::config::PlaybackPreset;
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
        return handle_help_key(key, ui_state);
    }

    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c'))
        || key.code == KeyCode::Char('q')
    {
        return Ok(true);
    }

    if handle_playback_keys(key.code, player, queue, ui_state)? {
        return Ok(false);
    }
    if handle_config_keys(key.code, player, queue, ui_state) {
        return Ok(false);
    }
    handle_modal_triggers(key.code, queue, ui_state);
    Ok(false)
}

fn handle_help_key(key: event::KeyEvent, ui_state: &mut UiState) -> Result<bool> {
    if matches!(
        key.code,
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
    ) {
        ui_state.show_help = false;
    }
    Ok(false)
}

fn handle_playback_keys(
    code: KeyCode,
    player: &mut ShadowPlayer,
    queue: &mut SentenceQueue,
    ui_state: &mut UiState,
) -> Result<bool> {
    match code {
        KeyCode::Char(' ') => {
            if let Err(e) = player.toggle_play(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(true)
        }
        KeyCode::Char('n') | KeyCode::Right => {
            if let Err(e) = player.next(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(true)
        }
        KeyCode::Char('p') | KeyCode::Left => {
            if let Err(e) = player.previous(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(true)
        }
        KeyCode::Char('r') => {
            if let Err(e) = player.replay(queue) {
                ui_state.set_status(format!("Audio error: {e}"));
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn handle_config_keys(
    code: KeyCode,
    player: &mut ShadowPlayer,
    queue: &mut SentenceQueue,
    ui_state: &mut UiState,
) -> bool {
    if handle_mode_and_gap_keys(code, player, ui_state) {
        return true;
    }
    if handle_display_and_volume_keys(code, player, ui_state) {
        return true;
    }
    handle_playback_settings_keys(code, player, queue, ui_state)
}

fn handle_mode_and_gap_keys(
    code: KeyCode,
    player: &mut ShadowPlayer,
    ui_state: &mut UiState,
) -> bool {
    match code {
        KeyCode::Char('m') | KeyCode::Tab => {
            let next_preset = player.config().preset.cycle();
            player.config_mut().preset = next_preset;
            ui_state.set_status(format!("Preset switched to [{}]", next_preset.name()));
            true
        }
        KeyCode::Char('[') => {
            adjust_gap(player, -0.2, ui_state);
            true
        }
        KeyCode::Char(']') => {
            adjust_gap(player, 0.2, ui_state);
            true
        }
        _ => false,
    }
}

fn handle_display_and_volume_keys(
    code: KeyCode,
    player: &mut ShadowPlayer,
    ui_state: &mut UiState,
) -> bool {
    match code {
        KeyCode::Char('t') => {
            player.config_mut().show_japanese = !player.config().show_japanese;
            let status = if player.config().show_japanese {
                "Visible"
            } else {
                "Hidden (Audio-First)"
            };
            ui_state.set_status(format!("Japanese text: {status}"));
            true
        }
        KeyCode::Char('f') => {
            player.config_mut().show_furigana = !player.config().show_furigana;
            true
        }
        KeyCode::Char('e') => {
            player.config_mut().show_translation = !player.config().show_translation;
            true
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            let cur = player.audio().volume();
            player.audio_mut().set_volume(cur + 0.05);
            player.config_mut().volume = player.audio().volume();
            ui_state.set_status(format!("Volume: {:.0}%", player.audio().volume() * 100.0));
            true
        }
        KeyCode::Char('-') => {
            let cur = player.audio().volume();
            player.audio_mut().set_volume(cur - 0.05);
            player.config_mut().volume = player.audio().volume();
            ui_state.set_status(format!("Volume: {:.0}%", player.audio().volume() * 100.0));
            true
        }
        _ => false,
    }
}

fn handle_playback_settings_keys(
    code: KeyCode,
    player: &mut ShadowPlayer,
    queue: &mut SentenceQueue,
    ui_state: &mut UiState,
) -> bool {
    match code {
        KeyCode::Char('s') => {
            queue.toggle_shuffle();
            let state_str = if queue.is_shuffled() { "ON" } else { "OFF" };
            ui_state.set_status(format!("Shuffle: {state_str}"));
            true
        }
        KeyCode::Char('a') => {
            player.config_mut().auto_advance = !player.config().auto_advance;
            let status = if player.config().auto_advance {
                "ON (auto-advance)"
            } else {
                "OFF (loop current sentence)"
            };
            ui_state.set_status(format!("Auto-advance: {status}"));
            true
        }
        KeyCode::Char(c) if ('1'..='5').contains(&c) => {
            let count = c.to_digit(10).unwrap_or(2);
            player.config_mut().repeat_count = count;
            ui_state.set_status(format!("Repetitions: {count}x per sentence"));
            true
        }
        _ => false,
    }
}

fn adjust_gap(player: &mut ShadowPlayer, delta: f32, ui_state: &mut UiState) {
    let preset = player.config().preset;
    match preset {
        PlaybackPreset::Shadow => {
            let new_gap = (player.config().shadow_breath_gap_secs + delta).clamp(0.4, 5.0);
            player.config_mut().shadow_breath_gap_secs = new_gap;
            ui_state.set_status(format!("Shadow breath gap: {new_gap:.1}s"));
        }
        PlaybackPreset::Echo => {
            let new_gap = (player.config().echo_breath_gap_secs + delta).clamp(0.4, 5.0);
            player.config_mut().echo_breath_gap_secs = new_gap;
            ui_state.set_status(format!("Echo breath gap: {new_gap:.1}s"));
        }
        PlaybackPreset::Listen => {
            let new_gap = (player.config().listen_gap_secs + delta).clamp(0.4, 5.0);
            player.config_mut().listen_gap_secs = new_gap;
            ui_state.set_status(format!("Listen spacing gap: {new_gap:.1}s"));
        }
    }
}

fn handle_modal_triggers(code: KeyCode, _queue: &mut SentenceQueue, ui_state: &mut UiState) {
    match code {
        KeyCode::Char('P') => {
            ui_state.show_playlists = true;
        }
        KeyCode::Char('/') => {
            ui_state.show_search = true;
            ui_state.search_query.clear();
        }
        KeyCode::Char('?') => {
            ui_state.show_help = true;
        }
        _ => {}
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
                ui_state.set_status(format!("Filter: \"{query}\" ({} found)", queue.total()));
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
        KeyCode::Esc | KeyCode::Char('q') => {
            ui_state.show_playlists = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if ui_state.playlist_cursor > 0 {
                ui_state.playlist_cursor -= 1;
            } else {
                ui_state.playlist_cursor = ui_state.available_playlists.len().saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if ui_state.playlist_cursor + 1 < ui_state.available_playlists.len() {
                ui_state.playlist_cursor += 1;
            } else {
                ui_state.playlist_cursor = 0;
            }
        }
        KeyCode::Enter => {
            if let Some(pl) = ui_state.available_playlists.get(ui_state.playlist_cursor) {
                queue.set_filter(pl.clone());
                ui_state.set_status(format!("Switched to: {}", pl.name()));
            }
            ui_state.show_playlists = false;
        }
        _ => {}
    }
    Ok(false)
}
