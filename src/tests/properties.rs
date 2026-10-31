use std::time::Duration;

use lofty::{
    file::{FileType, TaggedFile as LoftyTaggedFile},
    properties::FileProperties,
};

use crate::{
    music_file::{MusicFile, MusicFileInner},
    tests::music_file_from_path,
};

// ── audio properties ────────────────────────────────────────────────────

#[test]
fn test_mp3_properties() {
    let t = music_file_from_path("mp3.mp3");
    // MP3 is lossy: quality is HQ, no bit depth
    assert_eq!(t.quality(), "HQ");
    assert_eq!(t.bit_depth(), None);
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
    assert!(t.bit_rate().is_some());
}

#[test]
fn test_flac_properties() {
    let t = music_file_from_path("flac.flac");
    // FLAC is lossless: should have bit depth
    assert!(t.bit_depth().unwrap_or(0) > 0);
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
}

#[test]
fn test_ogg_properties() {
    let t = music_file_from_path("ogg.opus");
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
}

#[test]
fn test_wav_properties() {
    let t = music_file_from_path("wav.wav");
    // WAV is lossless: should have bit depth
    assert!(t.bit_depth().unwrap_or(0) > 0);
    assert!(t.sample_rate().unwrap_or(0) > 0);
    assert!(t.channels().unwrap_or(0) > 0);
}

#[test]
fn test_buffer_bitrate_falls_back_to_source_length() {
    let t = MusicFile::new_for_test(
        LoftyTaggedFile::new(
            FileType::Mpeg,
            FileProperties::new(Duration::from_secs(10), None, None, None, None, None, None),
            Vec::new(),
        ),
        MusicFileInner::Buffer {
            source_len: 160_000,
        },
    );

    assert_eq!(t.bit_rate().unwrap_or(0), 128);
}

#[test]
fn test_duration_does_not_wrap_at_u32_maximum() {
    let duration_ms = u64::from(u32::MAX) + 123;
    let t = MusicFile::new_for_test(
        LoftyTaggedFile::new(
            FileType::Mpeg,
            FileProperties::new(
                Duration::from_millis(duration_ms),
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            Vec::new(),
        ),
        MusicFileInner::Buffer { source_len: 0 },
    );

    assert_eq!(t.duration(), duration_ms as f64);
}

// ── quality ─────────────────────────────────────────────────────────────

fn music_file_with_format(
    file_type: FileType,
    sample_rate: Option<u32>,
    bit_depth: Option<u8>,
) -> MusicFile {
    MusicFile::new_for_test(
        LoftyTaggedFile::new(
            file_type,
            FileProperties::new(
                Duration::from_secs(10),
                None,
                None,
                sample_rate,
                bit_depth,
                None,
                None,
            ),
            Vec::new(),
        ),
        MusicFileInner::Buffer { source_len: 0 },
    )
}

#[test]
fn test_quality_lossless_cd_quality_is_sq() {
    let t = music_file_with_format(FileType::Flac, Some(44_100), Some(16));
    assert_eq!(t.quality(), "SQ");
}

#[test]
fn test_quality_16_bit_48khz_is_sq() {
    let t = music_file_with_format(FileType::Flac, Some(48_000), Some(16));
    assert_eq!(t.quality(), "SQ");
}

#[test]
fn test_quality_24_bit_44khz_is_hi_res() {
    let t = music_file_with_format(FileType::Flac, Some(44_100), Some(24));
    assert_eq!(t.quality(), "HiRes");
}

#[test]
fn test_quality_16_bit_96khz_is_hi_res() {
    let t = music_file_with_format(FileType::Wav, Some(96_000), Some(16));
    assert_eq!(t.quality(), "HiRes");
}

#[test]
fn test_quality_mp4_with_bit_depth_is_lossless() {
    // lofty only reports a bit depth for ALAC / FLAC streams in MP4
    let t = music_file_with_format(FileType::Mp4, Some(44_100), Some(16));
    assert_eq!(t.quality(), "SQ");

    let t = music_file_with_format(FileType::Mp4, Some(96_000), Some(24));
    assert_eq!(t.quality(), "HiRes");
}

#[test]
fn test_quality_mp4_without_bit_depth_is_lossy() {
    let t = music_file_with_format(FileType::Mp4, Some(44_100), None);
    assert_eq!(t.quality(), "HQ");
}
