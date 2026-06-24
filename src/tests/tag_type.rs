use crate::tests::tagged_file_from_path;

// ── tag type ────────────────────────────────────────────────────────────

#[test]
fn test_mp3_has_id3_tag() {
    let t = tagged_file_from_path("mp3.mp3");
    let tag_type = t.tag_type().unwrap();
    assert!(
        tag_type == "ID3V1" || tag_type == "ID3V2" || tag_type == "APE",
        "unexpected tag type: {tag_type}"
    );
}

#[test]
fn test_mp3_without_tag() {
    let t = tagged_file_from_path("mp3-no-tags.mp3");
    let tag_type = t.tag_type();
    assert!(tag_type.is_none(), "unexpected tag in file: {tag_type:?}")
}

#[test]
fn test_flac_has_vorbis_tag() {
    let t = tagged_file_from_path("flac.flac");
    assert_eq!(t.tag_type().as_deref(), Some("VORBIS"));
}

#[test]
fn test_ogg_has_vorbis_tag() {
    let t = tagged_file_from_path("ogg.opus");
    assert_eq!(t.tag_type().as_deref(), Some("VORBIS"));
}

#[test]
fn test_wav_has_riff_or_id3_tag() {
    let t = tagged_file_from_path("wav.wav");
    let tag_type = t.tag_type().unwrap();
    assert!(
        tag_type == "RIFF" || tag_type == "ID3V2",
        "unexpected tag type: {tag_type}"
    );
}
