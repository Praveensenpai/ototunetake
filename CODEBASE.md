# CODEBASE.md: Ototunetake Semantic Digest

> **Notice**: This file is an AI-optimized semantic index. Do not write narrative prose. Keep token density high.

## 1. System Topology & Data Flow

```text
Anki (SQLite collection.anki2 + collection.media)
  │ (Read-Only Scanner)
  ▼
Sentence Entities (domain::Sentence)
  │
  ├─► SentenceQueue (Filter, Shuffle, Nav)
  │
  ├─► ShadowPlayer State Machine (Reference Audio ─► Breathing Gap / Speaking Gap ─► Card Complete Break)
  │     │
  │     ├─► AudioEngine (rodio native playback + ffmpeg FLAC fallback)
  │     └─► PracticeStats (Session & daily persistence)
  │
  └─► Ratatui TUI (Header, PlayerCard, QueueView, Modals)
```

## 2. Global Constraints & Architecture Patterns

- **Language & Edition**: Rust 2021 edition.
- **Architectural Paradigm**: Role-based (`domain/`, `infra/`, `engine/`, `ui/`, `cli/`).
- **Zero Tolerance Invariants**: No `#[allow(...)]`, no production `.unwrap()` / `.expect()`, no `mod.rs` (modern folder.rs naming), 0 compiler/clippy warnings.
- **Complexity Hard Limits**: <400 lines/file (300 soft), <60 lines/fn (40 soft), max 4 parameters, max 3 nesting depth.
- **Target Distribution**: Linux x86_64 standalone binary.

## 3. Module & Interface Skeleton

### `src/error.rs` (Role: infra, Lines: 31)
- **Responsibility**: Centralized application error handling enum and Result type.
- **Types**:
  - `AppError`: `Database(rusqlite::Error)`, `Audio(String)`, `Io(std::io::Error)`, `Json(serde_json::Error)`, `CollectionNotFound(PathBuf)`, `MediaNotFound(PathBuf)`, `EmptyQueue`, `InvalidOperation(String)`.
  - `Result<T> = std::result::Result<T, AppError>`.

### `src/domain/sentence.rs` (Role: domain, Lines: 51)
- **Responsibility**: Core sentence entity and review card classification.
- **Types**:
  - `CardStatus`: `Mature`, `Young`, `New`, `Suspended`.
  - `Sentence`: `id: i64`, `card_id: i64`, `note_id: i64`, `japanese_text: String`, `furigana: Option<String>`, `translation: Option<String>`, `audio_path: PathBuf`, `deck: String`, `tags: Vec<String>`, `status: CardStatus`, `interval: i64`, `reps: i64`, `last_reviewed: Option<i64>`, `play_count: u64`, `shadow_count: u64`, `last_played: Option<i64>`.

### `src/domain/playlist.rs` (Role: domain, Lines: 85)
- **Responsibility**: Playlist filtering and sorting criteria.
- **Types**:
  - `PlaylistFilter`: `AllReviewed`, `Mature`, `Young`, `RecentlyReviewed`, `Deck(String)`, `Tag(String)`, `Search(String)`.
  - `Playlist`: `name: String`, `filter: PlaylistFilter`.

### `src/domain/config.rs` (Role: domain, Lines: 75)
- **Responsibility**: Player configuration and playback presets.
- **Types**:
  - `PlaybackPreset`: `Shadow`, `Echo`, `Listen`.
  - `AppConfig`: `preset`, `repeat_count`, `pause_after_replay_secs`, `auto_advance`, `show_japanese`, `show_furigana`, `show_translation`, `volume`, `anki_collection_path`, `anki_media_path`, `shadow_breath_gap_secs`, `echo_breath_gap_secs`, `card_complete_break_secs`, `listen_gap_secs`.

### `src/domain/stats.rs` (Role: domain, Lines: 56)
- **Responsibility**: Practice metrics tracking and daily rollover logic.
- **Types**:
  - `PracticeStats`: `total_listened_count`, `total_shadowed_count`, `total_listening_secs`, `today_date`, `today_sentences_count`, `today_repetitions_count`, `today_listening_secs`.

### `src/infra/anki.rs` (Role: infra, Lines: 275)
- **Responsibility**: Read-only SQLite Anki collection inspection and media file resolver.
- **Types**:
  - `AnkiPaths`: `collection_db: PathBuf`, `media_dir: PathBuf`.
  - `AnkiScanner`: `locate_default_paths() -> Result<AnkiPaths>`, `open_read_only(db_path: &Path) -> Result<Connection>`, `scan_reviewed_sentences(paths: &AnkiPaths) -> Result<Vec<Sentence>>`.

### `src/infra/audio.rs` (Role: infra, Lines: 241)
- **Responsibility**: Native low-latency audio playback engine using `rodio` with exact duration probing and lossless Opus/multi-codec fallback.
- **Types**:
  - `AudioReader`: `File(BufReader<File>)`, `Memory(Cursor<Vec<u8>>)`.
  - `AudioEngine`: `try_new() -> Result<Self>`, `play_file(path: &Path) -> Result<()>`, `toggle_pause()`, `is_playing() -> bool`, `is_paused() -> bool`, `is_done() -> bool`, `stop()`, `set_volume(vol: f32)`, `volume() -> f32`, `duration() -> Duration`, `progress_ratio() -> f64`, `elapsed() -> Duration`.

### `src/infra/storage.rs` (Role: infra, Lines: 102)
- **Responsibility**: Filesystem persistence for config, stats, and session state.
- **Types**:
  - `SessionState`: `last_sentence_id`, `playlist_name`, `volume`, `shuffle`.
  - `AppStorage`: `new() -> Result<Self>`, `load_config() -> AppConfig`, `save_config()`, `load_stats() -> PracticeStats`, `save_stats()`, `load_session() -> SessionState`, `save_session()`.

### `src/engine/phase.rs` (Role: engine, Lines: 130)
- **Responsibility**: Loop phase definitions and pause/resume duration preservation.
- **Types**:
  - `PlaybackStatus`: `Playing`, `Replaying`, `Paused`, `SpeakingGap`, `BreathingGap`, `CardComplete`, `Stopped`.
  - `ShadowLoopPhase`: `Stopped`, `PlayingReference`, `SpeakingGap`, `PlayingReplay`, `BreathingGap`, `CardCompleteBreak`, `Paused`.

### `src/engine/queue.rs` (Role: engine, Lines: 161)
- **Responsibility**: Active sentence queue, playlist filtering, and cursor navigation.
- **Types**:
  - `SentenceQueue`: `new(pool: Vec<Sentence>) -> Self`, `current()`, `advance()`, `previous()`, `jump_to()`, `toggle_shuffle()`, `set_filter()`, `remove_at()`.

### `src/engine/player.rs` (Role: engine, Lines: 330)
- **Responsibility**: Multi-phase Shadowing and Echo loop controller and auto-advancer.
- **Types**:
  - `ShadowPlayer`: `new()`, `phase()`, `status()`, `current_repetition()`, `play_current()`, `toggle_play()`, `replay()`, `next()`, `previous()`, `tick()`.

### `src/input.rs` (Role: ui/input, Lines: 250)
- **Responsibility**: Modular keyboard event dispatcher, volume/gap adjustments (`[` / `]`), and modal input handlers.
- **Functions**: `handle_key_event`, `handle_search_key`, `handle_playlist_key`.

### `src/ui/` (Role: ui)
- `theme.rs` (Lines: 13): Pastel palette matching Ototune aesthetic.
- `layout.rs` (Lines: 70): Split layout and bottom keybinding hints.
- `header.rs` (Lines: 155): Header with playback badges, preset status, repetition counter, and today's stats.
- `player_card.rs` (Lines: 285): Audio-first sentence display, phase-aware gauge progress with live timing, card review metadata.
- `queue_view.rs` (Lines: 75): Upcoming queue list and active item highlight.
- `modals.rs` (Lines: 250): Help modal (`?`), playlist selector (`P`), live search bar (`/`).

### `src/cli/args.rs` (Role: cli, Lines: 42)
- **Responsibility**: Clap CLI definitions for collection path, deck/tag filters, mode (`shadow`, `echo`, `listen`), and shuffle.
- **Types**: `CliArgs`.
