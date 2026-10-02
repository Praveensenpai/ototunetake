use ototunetake::infra::anki::AnkiScanner;
use ototunetake::infra::audio::AudioEngine;
use std::path::PathBuf;

#[test]
fn test_audio_engine_decode_opus_ogg() {
    let audio_path = PathBuf::from(
        "/home/paisen/.local/share/Anki2/User 1/collection.media/平仮名_ヒラガナ_TAAS(2).ogg",
    );
    if !audio_path.exists() {
        return;
    }

    let mut engine = AudioEngine::try_new().expect("Failed to create AudioEngine");
    let result = engine.play_file(&audio_path);
    assert!(
        result.is_ok(),
        "Failed to play Opus audio file: {:?}",
        result.err()
    );

    engine.stop();
}

#[test]
fn test_audio_engine_decode_all_756_reviewed_anki_sentences() {
    let paths = AnkiScanner::locate_default_paths().expect("Must locate Anki paths");
    let sentences = AnkiScanner::scan_reviewed_sentences(&paths).expect("Must scan sentences");
    assert!(!sentences.is_empty());

    let mut engine = AudioEngine::try_new().expect("Failed to create AudioEngine");
    let mut failed = Vec::new();

    for (idx, sentence) in sentences.iter().enumerate() {
        if let Err(e) = engine.play_file(&sentence.audio_path) {
            failed.push((idx, sentence.audio_path.clone(), format!("{e}")));
        }
        engine.stop();
    }

    if !failed.is_empty() {
        panic!(
            "Failed to decode {} files out of {}: {:?}",
            failed.len(),
            sentences.len(),
            failed
        );
    }

    println!(
        "SUCCESS: Verified decoding all {} reviewed audio files on disk!",
        sentences.len()
    );
}
