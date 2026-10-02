use ototunetake::domain::playlist::PlaylistFilter;
use ototunetake::domain::sentence::{CardStatus, Sentence};
use ototunetake::engine::queue::SentenceQueue;
use std::path::PathBuf;

fn create_sample_sentence(id: i64, text: &str, status: CardStatus, deck: &str) -> Sentence {
    Sentence {
        id,
        card_id: id,
        note_id: id,
        japanese_text: text.to_string(),
        furigana: None,
        translation: Some("English translation".to_string()),
        audio_path: PathBuf::from(format!("/tmp/test_{id}.ogg")),
        deck: deck.to_string(),
        tags: vec!["jp1k".to_string()],
        status,
        interval: match status {
            CardStatus::Mature => 30,
            CardStatus::Young => 10,
            _ => 0,
        },
        reps: 5,
        last_reviewed: Some(1728356495000),
        play_count: 0,
        shadow_count: 0,
        last_played: None,
    }
}

#[test]
fn test_queue_navigation() {
    let pool = vec![
        create_sample_sentence(1, "こんにちは", CardStatus::Mature, "Default"),
        create_sample_sentence(2, "さようなら", CardStatus::Young, "Default"),
        create_sample_sentence(3, "ありがとう", CardStatus::Mature, "Default"),
    ];

    let mut queue = SentenceQueue::new(pool);
    assert_eq!(queue.total(), 3);
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("こんにちは")
    );

    queue.advance();
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("さようなら")
    );

    queue.advance();
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("ありがとう")
    );

    queue.advance(); // Wrap around
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("こんにちは")
    );

    queue.previous(); // Wrap backwards
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("ありがとう")
    );
}

#[test]
fn test_queue_filtering() {
    let pool = vec![
        create_sample_sentence(1, "猫が好きです", CardStatus::Mature, "Animals"),
        create_sample_sentence(2, "犬が好きです", CardStatus::Young, "Animals"),
        create_sample_sentence(3, "林檎を食べる", CardStatus::Mature, "Food"),
    ];

    let mut queue = SentenceQueue::new(pool);
    assert_eq!(queue.total(), 3);

    queue.set_filter(PlaylistFilter::Mature);
    assert_eq!(queue.total(), 2);

    queue.set_filter(PlaylistFilter::Young);
    assert_eq!(queue.total(), 1);
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("犬が好きです")
    );

    queue.set_filter(PlaylistFilter::Deck("Food".to_string()));
    assert_eq!(queue.total(), 1);
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("林檎を食べる")
    );

    queue.set_filter(PlaylistFilter::Search("猫".to_string()));
    assert_eq!(queue.total(), 1);
    assert_eq!(
        queue.current().map(|s| s.japanese_text.as_str()),
        Some("猫が好きです")
    );
}
