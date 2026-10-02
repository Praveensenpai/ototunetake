pub mod header;
pub mod layout;
pub mod modals;
pub mod player_card;
pub mod queue_view;
pub mod theme;

use crate::domain::config::AppConfig;
use crate::domain::playlist::PlaylistFilter;
use crate::domain::stats::PracticeStats;
use crate::engine::player::ShadowPlayer;
use crate::engine::queue::SentenceQueue;
use ratatui::Frame;

pub struct UiState {
    pub show_help: bool,
    pub show_playlists: bool,
    pub show_search: bool,
    pub search_query: String,
    pub playlist_cursor: usize,
    pub available_playlists: Vec<PlaylistFilter>,
    pub status_message: Option<(String, std::time::Instant)>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_help: false,
            show_playlists: false,
            show_search: false,
            search_query: String::new(),
            playlist_cursor: 0,
            available_playlists: vec![
                PlaylistFilter::AllReviewed,
                PlaylistFilter::Mature,
                PlaylistFilter::Young,
                PlaylistFilter::RecentlyReviewed,
            ],
            status_message: None,
        }
    }
}

impl UiState {
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn current_status(&self) -> Option<&str> {
        if let Some((msg, time)) = &self.status_message {
            if time.elapsed() < std::time::Duration::from_secs(4) {
                return Some(msg.as_str());
            }
        }
        None
    }
}

pub fn render_app(
    frame: &mut Frame,
    player: &ShadowPlayer,
    queue: &SentenceQueue,
    stats: &PracticeStats,
    config: &AppConfig,
    ui_state: &UiState,
) {
    let chunks = layout::create_main_layout(frame.area());

    header::render_header(frame, player, queue, stats, chunks[0]);
    player_card::render_player_card(frame, player, queue, config, chunks[1]);
    queue_view::render_queue_view(frame, queue, chunks[2]);
    layout::render_footer(frame, ui_state, chunks[3]);

    if ui_state.show_help {
        modals::render_help_modal(frame);
    } else if ui_state.show_playlists {
        modals::render_playlist_modal(frame, ui_state);
    } else if ui_state.show_search {
        modals::render_search_modal(frame, ui_state);
    }
}
