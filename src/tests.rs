use std::path::PathBuf;

use crate::tagged_file::TaggedFile;

mod file;
mod meta_picture;
mod metadata;
mod properties;
mod tag_type;
mod utils;

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples")
}

fn tagged_file_from_path(name: &str) -> TaggedFile {
    let path = samples_dir().join(name).to_str().unwrap().to_string();
    TaggedFile::load_from_path_sync(path).expect("load_from_path failed")
}

fn tagged_file_from_buffer(name: &str) -> TaggedFile {
    let data: Vec<u8> = std::fs::read(samples_dir().join(name)).expect("read failed");
    TaggedFile::load_from_buffer(data.into()).expect("load_from_buffer failed")
}
