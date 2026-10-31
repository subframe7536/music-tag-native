use std::time::Duration;

use lofty::{
    file::{FileType, TaggedFile as LoftyTaggedFile, TaggedFileExt},
    picture::{MimeType, Picture, PictureType},
    properties::FileProperties,
    tag::{Accessor, ItemKey, Tag, TagType},
};
use napi::{bindgen_prelude::Null, Either};

use crate::{
    meta_picture::MetaPicture,
    music_file::{MusicFile, MusicFileInner},
};

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
    // The ID3v1 tag is kept in sync instead of keeping a stale value.
    assert_eq!(
        t.file.tag(TagType::Id3v1).unwrap().title().as_deref(),
        Some("Title")
    );
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

fn music_file_with_tags(file_type: FileType, tags: Vec<Tag>) -> MusicFile {
    MusicFile::new_for_test(
        LoftyTaggedFile::new(
            file_type,
            FileProperties::new(Duration::from_secs(1), None, None, None, None, None, None),
            tags,
        ),
        MusicFileInner::Buffer { source_len: 0 },
    )
}

// ── tag selection with ID3v1 next to another tag ────────────────────────

/// An MP3 with an ID3v1 tag listed before a richer APE tag
fn id3v1_and_ape_mp3() -> MusicFile {
    let mut id3v1 = Tag::new(TagType::Id3v1);
    id3v1.set_title("V1 title".to_string());
    let mut ape = Tag::new(TagType::Ape);
    ape.set_title("APE title".to_string());
    ape.insert_text(ItemKey::Composer, "APE composer".to_string());
    music_file_with_tags(FileType::Mpeg, vec![id3v1, ape])
}

#[test]
fn test_reads_prefer_other_tag_over_id3v1() {
    let t = id3v1_and_ape_mp3();

    assert_eq!(t.tag_type().as_deref(), Some("APE"));
    assert_eq!(t.title().as_deref(), Some("APE title"));
    assert_eq!(t.composer().as_deref(), Some("APE composer"));
}

#[test]
fn test_write_goes_to_other_tag_instead_of_id3v1() {
    let mut t = id3v1_and_ape_mp3();

    t.set_title(Either::A("New title".to_string())).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("APE"));
    assert!(t.file.tag(TagType::Id3v2).is_none());
    assert_eq!(t.title().as_deref(), Some("New title"));
    assert_eq!(
        t.file.tag(TagType::Id3v1).unwrap().title().as_deref(),
        Some("New title")
    );
}

#[test]
fn test_promotion_seeds_from_other_tag_instead_of_id3v1() {
    let mut t = id3v1_and_ape_mp3();

    // APE cannot store a rating, so an ID3v2 tag is created.
    t.set_rating(Either::A(4.0)).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(t.title().as_deref(), Some("APE title"));
    assert_eq!(t.composer().as_deref(), Some("APE composer"));
    assert_eq!(t.rating(), Some(4));
}

// ── lyrics while seeding ────────────────────────────────────────────────

#[test]
fn test_promotion_does_not_duplicate_lyrics() {
    let mut ape = Tag::new(TagType::Ape);
    ape.insert_text(ItemKey::Lyrics, "lyrics".to_string());
    ape.insert_text(ItemKey::UnsyncLyrics, "unsync lyrics".to_string());
    let mut t = music_file_with_tag(FileType::Mpeg, ape);

    t.set_rating(Either::A(4.0)).unwrap();

    let id3v2 = t.file.tag(TagType::Id3v2).unwrap();
    let lyrics: Vec<_> = id3v2
        .items()
        .filter(|item| matches!(item.key(), ItemKey::Lyrics | ItemKey::UnsyncLyrics))
        .collect();
    assert_eq!(lyrics.len(), 1, "unexpected lyrics: {lyrics:?}");
}

// ── clearing fields ─────────────────────────────────────────────────────

#[test]
fn test_clearing_field_without_tags_does_not_create_tag() {
    let mut t = music_file_with_tags(FileType::Mpeg, Vec::new());

    t.set_title(Either::B(napi::bindgen_prelude::Null)).unwrap();

    assert_eq!(t.tag_type(), None);
    assert!(t.file.tags().is_empty());
}

/// An MP3 whose only tag is ID3v1 with a title and an artist
fn id3v1_only_with_title_and_artist() -> MusicFile {
    let mut id3v1 = Tag::new(TagType::Id3v1);
    id3v1.set_title("Song".to_string());
    id3v1.set_artist("Artist".to_string());
    music_file_with_tag(FileType::Mpeg, id3v1)
}

#[test]
fn test_clearing_absent_field_on_id3v1_only_does_not_create_tag() {
    let mut t = id3v1_only_with_title_and_artist();

    t.set_rating(Either::B(napi::bindgen_prelude::Null))
        .unwrap();
    t.set_composer(Either::B(napi::bindgen_prelude::Null))
        .unwrap();
    t.set_pictures(Either::B(napi::bindgen_prelude::Null))
        .unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V1"));
    assert!(t.file.tag(TagType::Id3v2).is_none());
    assert_eq!(t.title().as_deref(), Some("Song"));
}

#[test]
fn test_clearing_existing_field_on_id3v1_only_edits_id3v1() {
    let mut t = id3v1_only_with_title_and_artist();

    t.set_title(Either::B(napi::bindgen_prelude::Null)).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V1"));
    assert!(t.file.tag(TagType::Id3v2).is_none());
    assert_eq!(t.title(), None);
    assert_eq!(t.artist().as_deref(), Some("Artist"));
}

#[test]
fn test_setting_value_id3v1_already_has_does_not_create_tag() {
    let mut t = id3v1_only_with_title_and_artist();

    // Nothing changes, so there is no reason to add an ID3v2 tag.
    t.set_title(Either::A("Song".to_string())).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V1"));
    assert!(t.file.tag(TagType::Id3v2).is_none());
    assert_eq!(t.title().as_deref(), Some("Song"));
}

// ── unsupported fields other than rating ────────────────────────────────

#[test]
fn test_unsupported_text_field_error_names_field_and_tag() {
    // The MP4 ilst map has a label key but no publisher key.
    let mut ilst = Tag::new(TagType::Mp4Ilst);
    ilst.set_title("Title".to_string());
    let mut t = music_file_with_tag(FileType::Mp4, ilst);

    let error = t
        .set_publisher(Either::A("Publisher".to_string()))
        .unwrap_err();

    assert_eq!(error.status, napi::Status::InvalidArg);
    assert_eq!(error.reason, "Publisher is not supported by the ILST tag");
    assert_eq!(t.publisher(), None);
    assert_eq!(t.title().as_deref(), Some("Title"));
    assert_eq!(t.file.tags().len(), 1);
}

// ── pictures while seeding and writing ──────────────────────────────────

fn picture(data: Vec<u8>, pic_type: PictureType) -> Picture {
    Picture::unchecked(data)
        .pic_type(pic_type)
        .mime_type(MimeType::Png)
        .build()
}

#[test]
fn test_promotion_copies_pictures() {
    let mut ape = Tag::new(TagType::Ape);
    ape.set_title("Title".to_string());
    ape.push_picture(picture(vec![1, 2, 3], PictureType::CoverFront));
    let mut t = music_file_with_tag(FileType::Mpeg, ape);

    // APE cannot store a rating, so an ID3v2 tag is created from the APE tag.
    t.set_rating(Either::A(4.0)).unwrap();

    assert_eq!(t.tag_type().as_deref(), Some("ID3V2"));
    let pictures = t.pictures().expect("pictures should be copied");
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].data.to_vec(), vec![1, 2, 3]);
    assert_eq!(pictures[0].cover_type, "Cover Art (Front)");
}

#[test]
fn test_set_pictures_shrinks_and_skips_id3v1() {
    let mut id3v2 = Tag::new(TagType::Id3v2);
    for data in [vec![1u8], vec![2], vec![3]] {
        id3v2.push_picture(picture(data, PictureType::Other));
    }
    let mut id3v1 = Tag::new(TagType::Id3v1);
    id3v1.set_title("Title".to_string());
    let mut t = music_file_with_tags(FileType::Mpeg, vec![id3v2, id3v1]);

    let replacement = MetaPicture::new("image/png".to_string(), vec![9u8].into(), None);
    t.set_pictures(Either::A(vec![&replacement])).unwrap();

    let pictures = t.pictures().unwrap();
    assert_eq!(pictures.len(), 1);
    assert_eq!(pictures[0].data.to_vec(), vec![9]);
    // ID3v1 cannot store pictures, so the sync leaves it without any.
    assert_eq!(t.file.tag(TagType::Id3v1).unwrap().picture_count(), 0);

    t.set_pictures(Either::B(Null)).unwrap();
    assert!(t.pictures().is_none());
}

// ── year: calendar rules and time ───────────────────────────────────────

fn file_with_recording_date(date: &str) -> MusicFile {
    let mut tag = Tag::new(TagType::Id3v2);
    tag.insert_text(ItemKey::RecordingDate, date.to_string());
    music_file_with_tag(FileType::Mpeg, tag)
}

#[test]
fn test_set_year_follows_century_leap_rules() {
    // 1900 is not a leap year (divisible by 100 but not by 400).
    let mut t = file_with_recording_date("2000-02-29");
    t.set_year(Either::A(1900.0)).unwrap();
    assert_eq!(recording_date(&t).as_deref(), Some("1900-02-28"));

    // 2000 is a leap year (divisible by 400), so Feb 29 is kept.
    let mut t = file_with_recording_date("1904-02-29");
    t.set_year(Either::A(2000.0)).unwrap();
    assert_eq!(recording_date(&t).as_deref(), Some("2000-02-29"));
}

#[test]
fn test_set_year_keeps_time() {
    let mut t = file_with_recording_date("2020-05-17T10:20:30");

    t.set_year(Either::A(2024.0)).unwrap();

    assert_eq!(recording_date(&t).as_deref(), Some("2024-05-17T10:20:30"));
}

#[test]
fn test_set_year_replaces_year_key() {
    // ID3v1 tags read from a file store the year under `ItemKey::Year`. The
    // full date goes to `RecordingDate`, so the old `Year` must not linger.
    let mut id3v1 = Tag::new(TagType::Id3v1);
    id3v1.insert_text(ItemKey::Year, "1999".to_string());
    let mut id3v2 = Tag::new(TagType::Id3v2);
    id3v2.insert_text(ItemKey::RecordingDate, "1999".to_string());
    let mut t = music_file_with_tags(FileType::Mpeg, vec![id3v2, id3v1]);

    t.set_year(Either::A(2024.0)).unwrap();

    let id3v1 = t.file.tag(TagType::Id3v1).unwrap();
    assert_eq!(id3v1.get_string(ItemKey::Year), None);
    assert_eq!(id3v1.get_string(ItemKey::RecordingDate), Some("2024"));
}
