use ototunetake::domain::config::{AppConfig, PlaybackPreset};
use ototunetake::domain::sentence::{CardStatus, Sentence};
use ototunetake::domain::stats::PracticeStats;
use ototunetake::engine::player::ShadowPlayer;
use ototunetake::engine::queue::SentenceQueue;
use ototunetake::infra::audio::AudioEngine;
use std::path::PathBuf;

fn create_dummy_sentence(id: i64) -> Sentence {
    Sentence {
        id,
        card_id: id,
        note_id: id,
        japanese_text: "テスト文です。".to_string(),
        furigana: None,
        translation: Some("This is a test sentence.".to_string()),
        audio_path: PathBuf::from("/dev/null"),
        deck: "Default".to_string(),
        tags: vec![],
        status: CardStatus::Mature,
        interval: 30,
        reps: 5,
        last_reviewed: Some(1700000000),
        play_count: 0,
        shadow_count: 0,
        last_played: None,
    }
}

#[test]
fn test_player_pause_preserves_remaining_gap() {
    let Ok(audio) = AudioEngine::try_new() else {
        return;
    };

    let config = AppConfig {
        preset: PlaybackPreset::Shadow,
        repeat_count: 2,
        shadow_breath_gap_secs: 1.5,
        card_complete_break_secs: 2.0,
        ..Default::default()
    };

    let mut player = ShadowPlayer::new(audio, config);
    let mut queue = SentenceQueue::new(vec![create_dummy_sentence(1)]);

    player.toggle_play(&mut queue).ok();
    player.stop();

    let mut stats = PracticeStats::default();
    let res = player.tick(&mut queue, &mut stats);
    assert!(res.is_ok());
}

#[test]
fn test_preset_names_and_cycles() {
    let p = PlaybackPreset::Shadow;
    assert_eq!(p.name(), "Shadow");
    let p = p.cycle();
    assert_eq!(p.name(), "Echo");
    let p = p.cycle();
    assert_eq!(p.name(), "Listen");
    let p = p.cycle();
    assert_eq!(p.name(), "Shadow");
}

#[test]
fn test_config_backward_compatibility() {
    let json_old = r#"{
        "preset": "Repeat",
        "repeat_count": 3,
        "pause_after_replay_secs": 1.0,
        "auto_advance": true,
        "show_japanese": true,
        "show_furigana": false,
        "show_translation": false,
        "volume": 0.8,
        "anki_collection_path": null,
        "anki_media_path": null
    }"#;

    let cfg: AppConfig = serde_json::from_str(json_old).expect("Must deserialize old config");
    assert_eq!(cfg.preset, PlaybackPreset::Echo);
    assert_eq!(cfg.shadow_breath_gap_secs, 1.2);
    assert_eq!(cfg.echo_breath_gap_secs, 1.0);
    assert_eq!(cfg.card_complete_break_secs, 2.0);
    assert_eq!(cfg.listen_gap_secs, 1.2);
}
