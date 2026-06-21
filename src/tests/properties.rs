use crate::tests::tagged_file_from_path;

// ── audio properties ────────────────────────────────────────────────────

#[test]
fn test_mp3_properties() {
    let t = tagged_file_from_path("mp3.mp3");
    // MP3 is lossy: quality is HQ, no bit depth
    assert_eq!(t.quality().unwrap(), "HQ");
    assert_eq!(t.bit_depth().unwrap(), None);
    assert!(t.sample_rate().unwrap().unwrap_or(0) > 0);
    assert!(t.channels().unwrap().unwrap_or(0) > 0);
    assert!(t.bit_rate().unwrap().is_some());
}

#[test]
fn test_flac_properties() {
    let t = tagged_file_from_path("flac.flac");
    // FLAC is lossless: should have bit depth
    assert!(t.bit_depth().unwrap().unwrap_or(0) > 0);
    assert!(t.sample_rate().unwrap().unwrap_or(0) > 0);
    assert!(t.channels().unwrap().unwrap_or(0) > 0);
}

#[test]
fn test_ogg_properties() {
    let t = tagged_file_from_path("ogg.opus");
    assert!(t.sample_rate().unwrap().unwrap_or(0) > 0);
    assert!(t.channels().unwrap().unwrap_or(0) > 0);
}

#[test]
fn test_wav_properties() {
    let t = tagged_file_from_path("wav.wav");
    // WAV is lossless: should have bit depth
    assert!(t.bit_depth().unwrap().unwrap_or(0) > 0);
    assert!(t.sample_rate().unwrap().unwrap_or(0) > 0);
    assert!(t.channels().unwrap().unwrap_or(0) > 0);
}
