use napi::{bindgen_prelude::Uint8Array, Either};

use crate::{
    music_file::MusicFile,
    tests::{music_file_from_buffer, music_file_from_path, samples_dir},
};

// ── load from path ─────────────────────────────────────────────────────

#[test]
fn test_load_mp3_from_path() {
    let t = music_file_from_path("mp3.mp3");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0);
}

#[test]
fn test_load_flac_from_path() {
    let t = music_file_from_path("flac.flac");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0);
}

#[test]
fn test_load_ogg_from_path() {
    let t = music_file_from_path("ogg.opus");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0);
}

#[test]
fn test_load_wav_from_path() {
    let t = music_file_from_path("wav.wav");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0);
}

#[test]
#[should_panic]
fn test_dont_load_garbage_file() {
    music_file_from_path("not-mp3.mp3");
}

// ── load from buffer ───────────────────────────────────────────────────

#[test]
fn test_load_mp3_from_buffer() {
    let t = music_file_from_buffer("mp3.mp3");
    assert!(t.tag_type().is_some());
}

#[test]
fn test_load_flac_from_buffer() {
    let t = music_file_from_buffer("flac.flac");
    assert!(t.tag_type().is_some());
}

#[test]
fn test_load_ogg_from_buffer() {
    let t = music_file_from_buffer("ogg.opus");
    assert!(t.tag_type().is_some());
}

#[test]
fn test_load_wav_from_buffer() {
    let t = music_file_from_buffer("wav.wav");
    assert!(t.tag_type().is_some());
}

#[test]
#[should_panic]
fn test_dont_load_garbage_buffer() {
    music_file_from_buffer("not-mp3.mp3");
}

// ── buffer round-trip save ──────────────────────────────────────────────

#[test]
fn test_mp3_buffer_save_round_trip() {
    let buffer: Vec<u8> = std::fs::read(samples_dir().join("mp3.mp3")).expect("read failed");
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&buffer)))
        .expect("load_sync failed");
    t.set_title(Either::A("Rust Test Title".to_string()))
        .unwrap();

    let Either::B(saved_buf) = t.save_sync(Some(Either::A(buffer.into()))).unwrap() else {
        panic!("save_sync did not return a buffer");
    };

    let t2 = MusicFile::load_sync(Either::A(saved_buf)).expect("load_sync failed");
    assert_eq!(t2.title().as_deref(), Some("Rust Test Title"));
}

#[test]
fn test_mp3_insert_new_tag_save_round_trip() {
    let buffer: Vec<u8> =
        std::fs::read(samples_dir().join("mp3-no-tags.mp3")).expect("read failed");
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&buffer)))
        .expect("load_sync failed");
    t.set_title(Either::A("Rust Test Title".to_string()))
        .unwrap();

    let Either::B(saved_buf) = t.save_sync(Some(Either::A(buffer.into()))).unwrap() else {
        panic!("save_sync did not return a buffer");
    };

    let t2 = MusicFile::load_sync(Either::A(saved_buf)).expect("load_sync failed");
    assert_eq!(t2.title().as_deref(), Some("Rust Test Title"));
}

#[test]
fn test_flac_buffer_save_round_trip() {
    let buffer: Vec<u8> = std::fs::read(samples_dir().join("flac.flac")).expect("read failed");
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&buffer)))
        .expect("load_sync failed");
    t.set_title(Either::A("FLAC Rust Title".to_string()))
        .unwrap();

    let Either::B(saved_buf) = t.save_sync(Some(Either::A(buffer.into()))).unwrap() else {
        panic!("save_sync did not return a buffer");
    };

    let t2 = MusicFile::load_sync(Either::A(saved_buf)).expect("load_sync failed");
    assert_eq!(t2.title().as_deref(), Some("FLAC Rust Title"));
}

#[test]
fn test_ogg_buffer_save_round_trip() {
    let buffer: Vec<u8> = std::fs::read(samples_dir().join("ogg.opus")).expect("read failed");
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&buffer)))
        .expect("load_sync failed");
    t.set_title(Either::A("OGG Rust Title".to_string()))
        .unwrap();

    let Either::B(saved_buf) = t.save_sync(Some(Either::A(buffer.into()))).unwrap() else {
        panic!("save_sync did not return a buffer");
    };

    let t2 = MusicFile::load_sync(Either::A(saved_buf)).expect("load_sync failed");
    assert_eq!(t2.title().as_deref(), Some("OGG Rust Title"));
}
