use std::path::PathBuf;

use napi::Either;

use crate::music_file::MusicFile;

mod file;
mod meta_picture;
mod metadata;
mod properties;
mod tag_type;
mod utils;

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples")
}

fn music_file_from_path(name: &str) -> MusicFile {
    let path = samples_dir().join(name).to_str().unwrap().to_string();
    MusicFile::load_sync(Either::B(path)).expect("load failed")
}

fn music_file_from_buffer(name: &str) -> MusicFile {
    let data: Vec<u8> = std::fs::read(samples_dir().join(name)).expect("read failed");
    MusicFile::load_sync(Either::A(data.into())).expect("load_sync failed")
}
