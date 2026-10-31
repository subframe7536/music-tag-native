use std::time::Duration;

use lofty::{
    file::{FileType, TaggedFile as LoftyTaggedFile, TaggedFileExt},
    properties::FileProperties,
    tag::{Tag, TagType},
};
use napi::Either;

use crate::music_file::{MusicFile, MusicFileInner};

fn music_file_with_tag(file_type: FileType, tag: Tag) -> MusicFile {
    MusicFile::new_for_test(
        LoftyTaggedFile::new(
            file_type,
            FileProperties::new(Duration::from_secs(1), None, None, None, None, None, None),
            vec![tag],
        ),
        MusicFileInner::Buffer { source_len: 0 },
    )
}

// ── unsupported fields ──────────────────────────────────────────────────

#[test]
fn test_unsupported_field_returns_error() {
    let mut t = music_file_with_tag(FileType::Ape, Tag::new(TagType::Ape));

    let error = t.set_rating(Either::A(3.0)).unwrap_err();
    assert_eq!(error.status, napi::Status::InvalidArg);
    assert_eq!(error.reason, "Rating is not supported by the APE tag");
    assert_eq!(t.rating(), None);
}

#[test]
fn test_supported_field_on_same_tag_still_works() {
    let mut t = music_file_with_tag(FileType::Ape, Tag::new(TagType::Ape));

    t.set_composer(Either::A("Composer".to_string())).unwrap();
    assert_eq!(t.composer().as_deref(), Some("Composer"));
}

#[test]
fn test_removing_unsupported_field_is_ok() {
    let mut t = music_file_with_tag(FileType::Ape, Tag::new(TagType::Ape));

    assert!(t.set_rating(Either::B(napi::bindgen_prelude::Null)).is_ok());
}

// ── secondary tag handling ──────────────────────────────────────────────

#[test]
fn test_failed_setter_does_not_create_tag() {
    let mut t = MusicFile::new_for_test(
        LoftyTaggedFile::new(
            FileType::Ape,
            FileProperties::new(Duration::from_secs(1), None, None, None, None, None, None),
            Vec::new(),
        ),
        MusicFileInner::Buffer { source_len: 0 },
    );

    assert!(t.set_rating(Either::A(3.0)).is_err());
    assert_eq!(t.tag_type(), None);
    assert!(t.file.tags().is_empty());
}
