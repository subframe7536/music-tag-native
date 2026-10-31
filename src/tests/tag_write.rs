use std::time::Duration;

use lofty::{
    file::{FileType, TaggedFile as LoftyTaggedFile, TaggedFileExt},
    properties::FileProperties,
    tag::{Accessor, ItemKey, Tag, TagType},
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

fn recording_date(file: &MusicFile) -> Option<String> {
    file.file
        .primary_tag()?
        .get_string(ItemKey::RecordingDate)
        .map(String::from)
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

// ── primary tag promotion ───────────────────────────────────────────────

#[test]
fn test_write_creates_primary_tag_seeded_from_secondary() {
    let mut id3v1 = Tag::new(TagType::Id3v1);
    id3v1.set_title("Old title".to_string());
    id3v1.set_artist("Old artist".to_string());
    let mut t = music_file_with_tag(FileType::Mpeg, id3v1);
    assert_eq!(t.tag_type().as_deref(), Some("ID3V1"));

    // ID3v1 cannot store a composer, the new ID3v2 primary tag can.
    t.set_composer(Either::A("Composer".to_string())).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(t.composer().as_deref(), Some("Composer"));
    assert_eq!(t.title().as_deref(), Some("Old title"));
    assert_eq!(t.artist().as_deref(), Some("Old artist"));
    // The secondary tag is kept as-is.
    assert!(t.file.tag(TagType::Id3v1).is_some());
}

#[test]
fn test_lyrics_use_key_of_written_tag() {
    let mut t = music_file_with_tag(FileType::Mpeg, Tag::new(TagType::Id3v1));

    t.set_lyrics(Either::A("Line 1\nLine 2".to_string()))
        .unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(t.lyrics().as_deref(), Some("Line 1\nLine 2"));
}

// ── year ────────────────────────────────────────────────────────────────

#[test]
fn test_set_year_keeps_month_and_day() {
    let mut tag = Tag::new(TagType::Id3v2);
    tag.insert_text(ItemKey::RecordingDate, "2020-05-17".to_string());
    let mut t = music_file_with_tag(FileType::Mpeg, tag);

    t.set_year(Either::A(2024.0)).unwrap();

    assert_eq!(t.year(), Some(2024));
    assert_eq!(recording_date(&t).as_deref(), Some("2024-05-17"));
}

#[test]
fn test_set_year_clamps_leap_day() {
    let mut tag = Tag::new(TagType::Id3v2);
    tag.insert_text(ItemKey::RecordingDate, "2020-02-29".to_string());
    let mut t = music_file_with_tag(FileType::Mpeg, tag);

    t.set_year(Either::A(2023.0)).unwrap();
    assert_eq!(recording_date(&t).as_deref(), Some("2023-02-28"));

    t.set_year(Either::A(2024.0)).unwrap();
    assert_eq!(recording_date(&t).as_deref(), Some("2024-02-28"));
}

#[test]
fn test_set_year_without_existing_date() {
    let mut t = music_file_with_tag(FileType::Mpeg, Tag::new(TagType::Id3v2));

    t.set_year(Either::A(1999.0)).unwrap();

    assert_eq!(t.year(), Some(1999));
    assert_eq!(recording_date(&t).as_deref(), Some("1999"));
}

// ── secondary tag handling ──────────────────────────────────────────────

#[test]
fn test_supported_field_writes_to_existing_secondary_tag() {
    let mut t = music_file_with_tag(FileType::Mpeg, Tag::new(TagType::Ape));

    t.set_title(Either::A("Title".to_string())).unwrap();

    // APE can store a title, so no ID3v2 tag is created.
    assert_eq!(t.tag_type().as_deref(), Some("APE"));
    assert!(t.file.tag(TagType::Id3v2).is_none());
    assert_eq!(t.title().as_deref(), Some("Title"));
}

#[test]
fn test_id3v1_write_always_creates_primary_tag() {
    let mut id3v1 = Tag::new(TagType::Id3v1);
    id3v1.set_artist("Old artist".to_string());
    let mut t = music_file_with_tag(FileType::Mpeg, id3v1);

    // ID3v1 maps a title, but truncates it to 30 bytes on save.
    t.set_title(Either::A("Title".to_string())).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(t.title().as_deref(), Some("Title"));
    assert_eq!(t.artist().as_deref(), Some("Old artist"));
    assert_eq!(t.file.tag(TagType::Id3v1).unwrap().title().as_deref(), None);
}

#[test]
fn test_promotion_keeps_ape_lyrics_readable() {
    let mut ape = Tag::new(TagType::Ape);
    ape.insert_text(ItemKey::Lyrics, "hello".to_string());
    let mut t = music_file_with_tag(FileType::Mpeg, ape);
    assert_eq!(t.lyrics().as_deref(), Some("hello"));

    // APE cannot store a rating, so an ID3v2 tag is created from the APE tag.
    t.set_rating(Either::A(4.0)).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(t.rating(), Some(4));
    assert_eq!(t.lyrics().as_deref(), Some("hello"));
}

#[test]
fn test_promotion_keeps_multi_valued_items() {
    let mut ape = Tag::new(TagType::Ape);
    ape.push(lofty::tag::TagItem::new(
        ItemKey::TrackArtist,
        lofty::tag::ItemValue::Text("A".to_string()),
    ));
    ape.push(lofty::tag::TagItem::new(
        ItemKey::TrackArtist,
        lofty::tag::ItemValue::Text("B".to_string()),
    ));
    let mut t = music_file_with_tag(FileType::Mpeg, ape);

    t.set_rating(Either::A(4.0)).unwrap();

    let primary = t.file.primary_tag().unwrap();
    assert_eq!(primary.get_strings(ItemKey::TrackArtist).count(), 2);
}

#[test]
fn test_failed_setter_does_not_change_secondary_only_file() {
    // An APE file with only an ID3v1 tag: neither ID3v1 nor APE stores ratings.
    let mut t = music_file_with_tag(FileType::Ape, Tag::new(TagType::Id3v1));

    let error = t.set_rating(Either::A(3.0)).unwrap_err();

    // ID3v1 is skipped, so the error is for the APE tag that would be created.
    assert_eq!(error.reason, "Rating is not supported by the APE tag");
    assert_eq!(t.tag_type().as_deref(), Some("ID3V1"));
    assert!(t.file.tag(TagType::Ape).is_none());
}

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
