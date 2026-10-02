use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ototunetake::cli::CliArgs;
use ototunetake::domain::config::{AppConfig, PlaybackPreset};
use ototunetake::domain::playlist::PlaylistFilter;
use ototunetake::engine::player::ShadowPlayer;
use ototunetake::engine::queue::SentenceQueue;
use ototunetake::error::{AppError, Result};
use ototunetake::infra::anki::{AnkiPaths, AnkiScanner};
use ototunetake::infra::audio::AudioEngine;
use ototunetake::infra::storage::{AppStorage, SessionState};
use ototunetake::input;
use ototunetake::ui::{self, UiState};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::collections::HashSet;
use std::io;
use std::time::Duration;

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let storage = AppStorage::new()?;
    let mut config = storage.load_config();
    let mut stats = storage.load_stats();
    let session = storage.load_session();

    apply_cli_overrides(&args, &mut config);

    let paths = resolve_anki_paths(&args, &config)?;
    let sentences = AnkiScanner::scan_reviewed_sentences(&paths)?;

    if sentences.is_empty() {
        return Err(AppError::InvalidOperation(
            "No reviewed cards with audio found in your Anki collection.".to_string(),
        ));
    }

    let mut ui_state = UiState::default();
    populate_available_playlists(&sentences, &mut ui_state);

    let mut queue = SentenceQueue::new(sentences);
    apply_initial_queue_state(&args, &session, &mut queue, &ui_state);

    let mut audio = AudioEngine::try_new()?;
    audio.set_volume(config.volume);
    let mut player = ShadowPlayer::new(audio, config);

    run_terminal_app(&mut player, &mut queue, &mut stats, &mut ui_state, &storage)?;

    Ok(())
}

fn apply_cli_overrides(args: &CliArgs, config: &mut AppConfig) {
    if args.show_text {
        config.show_japanese = true;
    }
    match args.mode.to_lowercase().as_str() {
        "listen" => config.preset = PlaybackPreset::Listen,
        "repeat" => config.preset = PlaybackPreset::Repeat,
        "shadow" => config.preset = PlaybackPreset::Shadow,
        _ => {}
    }
}

fn resolve_anki_paths(args: &CliArgs, config: &AppConfig) -> Result<AnkiPaths> {
    if let (Some(db), Some(media)) = (&args.collection, &args.media) {
        return Ok(AnkiPaths {
            collection_db: db.clone(),
            media_dir: media.clone(),
        });
    }

    if let (Some(db), Some(media)) = (&config.anki_collection_path, &config.anki_media_path) {
        if db.exists() && media.exists() {
            return Ok(AnkiPaths {
                collection_db: db.clone(),
                media_dir: media.clone(),
            });
        }
    }

    AnkiScanner::locate_default_paths()
}

fn populate_available_playlists(
    sentences: &[ototunetake::domain::sentence::Sentence],
    ui_state: &mut UiState,
) {
    let mut decks = HashSet::new();
    let mut tags = HashSet::new();

    for s in sentences {
        decks.insert(s.deck.clone());
        for t in &s.tags {
            tags.insert(t.clone());
        }
    }

    let mut sorted_decks: Vec<String> = decks.into_iter().collect();
    sorted_decks.sort();
    for d in sorted_decks {
        ui_state.available_playlists.push(PlaylistFilter::Deck(d));
    }

    let mut sorted_tags: Vec<String> = tags.into_iter().collect();
    sorted_tags.sort();
    for t in sorted_tags {
        ui_state.available_playlists.push(PlaylistFilter::Tag(t));
    }
}

fn apply_initial_queue_state(
    args: &CliArgs,
    session: &SessionState,
    queue: &mut SentenceQueue,
    ui_state: &UiState,
) {
    if args.mature {
        queue.set_filter(PlaylistFilter::Mature);
    } else if args.young {
        queue.set_filter(PlaylistFilter::Young);
    } else if let Some(d) = &args.deck {
        queue.set_filter(PlaylistFilter::Deck(d.clone()));
    } else if let Some(t) = &args.tag {
        queue.set_filter(PlaylistFilter::Tag(t.clone()));
    } else if let Some(filter) = ui_state
        .available_playlists
        .iter()
        .find(|pl| pl.name() == session.playlist_name)
    {
        queue.set_filter(filter.clone());
    }

    if args.shuffle || session.shuffle {
        queue.toggle_shuffle();
    }

    if let Some(last_id) = session.last_sentence_id {
        if let Some(pos) = queue
            .active_sentences()
            .iter()
            .position(|s| s.id == last_id)
        {
            queue.jump_to(pos);
        }
    }
}

fn run_terminal_app(
    player: &mut ShadowPlayer,
    queue: &mut SentenceQueue,
    stats: &mut ototunetake::domain::stats::PracticeStats,
    ui_state: &mut UiState,
    storage: &AppStorage,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = main_loop(&mut terminal, player, queue, stats, ui_state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    persist_session(player, queue, stats, storage);

    res
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    player: &mut ShadowPlayer,
    queue: &mut SentenceQueue,
    stats: &mut ototunetake::domain::stats::PracticeStats,
    ui_state: &mut UiState,
) -> Result<()> {
    let tick_rate = Duration::from_millis(50);

    loop {
        terminal.draw(|f| {
            ui::render_app(f, player, queue, stats, player.config(), ui_state);
        })?;

        player.tick(queue, stats)?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if input::handle_key_event(key, player, queue, ui_state)? {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn persist_session(
    player: &ShadowPlayer,
    queue: &SentenceQueue,
    stats: &ototunetake::domain::stats::PracticeStats,
    storage: &AppStorage,
) {
    let session = SessionState {
        last_sentence_id: queue.current().map(|s| s.id),
        playlist_name: queue.active_filter().name(),
        volume: player.audio().volume(),
        shuffle: queue.is_shuffled(),
    };

    let _ = storage.save_session(&session);
    let _ = storage.save_config(player.config());
    let _ = storage.save_stats(stats);
}
