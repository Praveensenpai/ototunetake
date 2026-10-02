use ototunetake::infra::anki::AnkiScanner;

#[test]
fn test_anki_auto_discovery_and_scan() {
    let paths = AnkiScanner::locate_default_paths();
    assert!(
        paths.is_ok(),
        "Expected to locate default Anki paths on system"
    );

    let paths = paths.unwrap();
    assert!(paths.collection_db.exists());
    assert!(paths.media_dir.exists());

    let sentences = AnkiScanner::scan_reviewed_sentences(&paths);
    assert!(sentences.is_ok(), "Expected scan to succeed");

    let sentences = sentences.unwrap();
    assert!(
        !sentences.is_empty(),
        "Expected reviewed sentences to be found"
    );

    // Verify all found sentences have existing audio files
    for s in &sentences {
        assert!(
            s.audio_path.exists(),
            "Audio file must exist: {:?}",
            s.audio_path
        );
        assert!(
            !s.japanese_text.is_empty(),
            "Sentence must have Japanese text"
        );
    }

    println!(
        "Scanned {} valid reviewed sentences with confirmed audio.",
        sentences.len()
    );
}
