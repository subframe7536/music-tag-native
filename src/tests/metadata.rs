use crate::tests::tagged_file_from_path;

// ── metadata read ───────────────────────────────────────────────────────

#[test]
fn test_mp3_title_read() {
    let t = tagged_file_from_path("mp3.mp3");
    // title can be Some or None, but reading must not panic
    let _ = t.title().unwrap();
}

#[test]
fn test_flac_title_read() {
    let t = tagged_file_from_path("flac.flac");
    let _ = t.title().unwrap();
}

#[test]
fn test_ogg_title_read() {
    let t = tagged_file_from_path("ogg.opus");
    let _ = t.title().unwrap();
}

#[test]
fn test_wav_title_read() {
    let t = tagged_file_from_path("wav.wav");
    let _ = t.title().unwrap();
}
