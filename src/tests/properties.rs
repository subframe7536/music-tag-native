use std::time::Duration;

use lofty::{
    file::{FileType, TaggedFile as LoftyTaggedFile},
    properties::FileProperties,
};

use crate::{
    tagged_file::{TaggedFile, TaggedFileInner},
    tests::tagged_file_from_path,
};

// ── audio properties ────────────────────────────────────────────────────

#[test]
fn test_mp3_properties() {
    let t = tagged_file_from_path("mp3.mp3");
    // MP3 is lossy: quality is HQ, no bit depth
    assert_eq!(t.quality(), "HQ");
    assert_eq!(t.bit_depth(), None);
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
    assert!(t.bit_rate().is_some());
}

#[test]
fn test_flac_properties() {
    let t = tagged_file_from_path("flac.flac");
    // FLAC is lossless: should have bit depth
    assert!(t.bit_depth().unwrap_or(0) > 0);
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
}

#[test]
fn test_ogg_properties() {
    let t = tagged_file_from_path("ogg.opus");
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
}

#[test]
fn test_wav_properties() {
    let t = tagged_file_from_path("wav.wav");
    // WAV is lossless: should have bit depth
    assert!(t.bit_depth().unwrap_or(0) > 0);
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
}

#[test]
fn test_buffer_bitrate_falls_back_to_source_length() {
    let t = TaggedFile::new_for_test(
        LoftyTaggedFile::new(
            FileType::Mpeg,
            FileProperties::new(Duration::from_secs(10), None, None, None, None, None, None),
            Vec::new(),
        ),
        TaggedFileInner::Buffer {
            source_len: 160_000,
        },
    );

    assert_eq!(t.bit_rate().unwrap_or(0), 128);
}
