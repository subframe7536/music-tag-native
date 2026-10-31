use napi::{
    bindgen_prelude::{Null, Uint8Array},
    Either,
};

use crate::{
    meta_picture::MetaPicture,
    music_file::MusicFile,
    tests::{music_file_from_buffer, music_file_from_path, samples_dir},
};

// ── load from path ─────────────────────────────────────────────────────

#[test]
fn test_load_mp3_from_path() {
    let t = music_file_from_path("mp3.mp3");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0.0);
}

#[test]
fn test_load_flac_from_path() {
    let t = music_file_from_path("flac.flac");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0.0);
}

#[test]
fn test_load_ogg_from_path() {
    let t = music_file_from_path("ogg.opus");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0.0);
}

#[test]
fn test_load_wav_from_path() {
    let t = music_file_from_path("wav.wav");
    assert!(t.tag_type().is_some());
    assert!(t.duration() > 0.0);
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

/// An MP3 whose only tag is an ID3v1 tag with the title "V1 Title"
fn id3v1_only_mp3() -> Vec<u8> {
    let mut buffer = std::fs::read(samples_dir().join("mp3-no-tags.mp3")).expect("read failed");
    let mut tag = [0u8; 128];
    tag[..3].copy_from_slice(b"TAG");
    tag[3..11].copy_from_slice(b"V1 Title");
    tag[127] = 255; // no genre
    buffer.extend_from_slice(&tag);
    buffer
}

fn save_and_reload(t: &MusicFile, source: Vec<u8>) -> MusicFile {
    let Either::B(saved_buf) = t.save_sync(Some(Either::A(source.into()))).unwrap() else {
        panic!("save_sync did not return a buffer");
    };
    MusicFile::load_sync(Either::A(saved_buf)).expect("load_sync failed")
}

#[test]
fn test_id3v1_only_mp3_keeps_pictures_after_save() {
    let source = id3v1_only_mp3();
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&source))).unwrap();
    assert_eq!(t.tag_type().as_deref(), Some("ID3V1"));

    let picture = MetaPicture::new("image/png".to_string(), vec![1u8, 2, 3, 4].into(), None);
    t.set_pictures(Either::A(vec![&picture])).unwrap();

    let reloaded = save_and_reload(&t, source);
    assert_eq!(reloaded.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(reloaded.pictures().map(|pictures| pictures.len()), Some(1));
    assert_eq!(reloaded.title().as_deref(), Some("V1 Title"));
}

#[test]
fn test_id3v1_only_mp3_keeps_long_title_after_save() {
    let source = id3v1_only_mp3();
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&source))).unwrap();
    let title = "A".repeat(40);

    t.set_title(Either::A(title.clone())).unwrap();

    let reloaded = save_and_reload(&t, source);
    assert_eq!(reloaded.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(reloaded.title(), Some(title));
}

/// Title stored in the trailing ID3v1 tag of `buffer`, or `None` without a tag
fn id3v1_title(buffer: &[u8]) -> Option<String> {
    let tag = &buffer[buffer.len() - 128..];
    if &tag[..3] != b"TAG" {
        return None;
    }
    Some(
        String::from_utf8_lossy(&tag[3..33])
            .trim_end_matches('\0')
            .to_string(),
    )
}

fn save_to_new_buffer(t: &MusicFile, source: Vec<u8>) -> Vec<u8> {
    let Either::B(saved_buf) = t.save_sync(Some(Either::A(source.into()))).unwrap() else {
        panic!("save_sync did not return a buffer");
    };
    saved_buf.to_vec()
}

#[test]
fn test_id3v1_only_mp3_updates_id3v1_on_write() {
    let source = id3v1_only_mp3();
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&source))).unwrap();

    t.set_title(Either::A("New Title".to_string())).unwrap();

    let saved = save_to_new_buffer(&t, source);
    assert_eq!(id3v1_title(&saved).as_deref(), Some("New Title"));
    let reloaded = MusicFile::load_sync(Either::A(saved.into())).unwrap();
    assert_eq!(reloaded.tag_type().as_deref(), Some("ID3V2"));
    assert_eq!(reloaded.title().as_deref(), Some("New Title"));
}

#[test]
fn test_id3v1_only_mp3_removed_field_stays_removed() {
    let source = id3v1_only_mp3();
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&source))).unwrap();

    t.set_title(Either::B(Null)).unwrap();

    let saved = save_to_new_buffer(&t, source);
    // The title was the only ID3v1 field, so the emptied tag is not written.
    assert_eq!(id3v1_title(&saved), None);
    let reloaded = MusicFile::load_sync(Either::A(saved.into())).unwrap();
    assert_eq!(reloaded.title(), None);
}

#[test]
fn test_id3v2_and_id3v1_mp3_updates_both_tags() {
    let source = id3v1_only_mp3();
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&source))).unwrap();
    t.set_title(Either::A("First".to_string())).unwrap();
    let with_both = save_to_new_buffer(&t, source);

    // Now the file has both tags, so writes go to the existing ID3v2 tag.
    let mut t = MusicFile::load_sync(Either::A(with_both.clone().into())).unwrap();
    assert_eq!(t.tag_type().as_deref(), Some("ID3V2"));
    t.set_title(Either::A("Second".to_string())).unwrap();
    // A field ID3v1 cannot store still succeeds.
    t.set_album_artist(Either::A("Album Artist".to_string()))
        .unwrap();

    let saved = save_to_new_buffer(&t, with_both);
    assert_eq!(id3v1_title(&saved).as_deref(), Some("Second"));
    let reloaded = MusicFile::load_sync(Either::A(saved.into())).unwrap();
    assert_eq!(reloaded.title().as_deref(), Some("Second"));
    assert_eq!(reloaded.album_artist().as_deref(), Some("Album Artist"));
}

// ── path save ───────────────────────────────────────────────────────────

/// Copy a sample into a fresh temporary directory and return (dir, path)
fn copy_sample_to_temp(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir failed");
    let path = dir.path().join(name);
    std::fs::copy(samples_dir().join(name), &path).expect("copy failed");
    (dir, path)
}

fn load_path(path: &std::path::Path) -> MusicFile {
    MusicFile::load_sync(Either::B(path.to_str().unwrap().to_string())).expect("load failed")
}

fn temp_entries(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".music-tag-native-"))
        .collect()
}

#[test]
fn test_in_place_save_round_trip() {
    let (dir, path) = copy_sample_to_temp("mp3.mp3");
    let mut t = load_path(&path);
    t.set_title(Either::A("In place".to_string())).unwrap();

    t.save_sync(None).unwrap();

    assert_eq!(load_path(&path).title().as_deref(), Some("In place"));
    assert!(temp_entries(dir.path()).is_empty());
}

#[test]
fn test_path_getter_returns_path_as_passed() {
    let (_dir, path) = copy_sample_to_temp("mp3.mp3");
    let path = path.to_str().unwrap().to_string();
    let t = MusicFile::load_sync(Either::B(path.clone())).unwrap();

    assert_eq!(t.path(), Some(&path));
}

#[cfg(unix)]
#[test]
fn test_in_place_save_replaces_file_atomically() {
    use std::os::unix::fs::MetadataExt;

    let (_dir, path) = copy_sample_to_temp("mp3.mp3");
    let inode_before = std::fs::metadata(&path).unwrap().ino();
    let mut t = load_path(&path);
    t.set_title(Either::A("Atomic".to_string())).unwrap();

    t.save_sync(None).unwrap();

    // Writing through a temporary file and renaming it gives a new inode.
    assert_ne!(std::fs::metadata(&path).unwrap().ino(), inode_before);
    assert_eq!(load_path(&path).title().as_deref(), Some("Atomic"));
}

#[cfg(unix)]
#[test]
fn test_custom_path_save_keeps_destination_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let dest = dir.path().join("dest.mp3");
    std::fs::copy(&src, &dest).unwrap();
    std::fs::set_permissions(&src, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o640)).unwrap();

    let mut t = load_path(&src);
    t.set_title(Either::A("Permissions".to_string())).unwrap();
    t.save_sync(Some(Either::B(dest.to_str().unwrap().to_string())))
        .unwrap();

    let mode = std::fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
    assert_eq!(load_path(&dest).title().as_deref(), Some("Permissions"));
}

fn set_readonly(path: &std::path::Path, readonly: bool) {
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(readonly);
    std::fs::set_permissions(path, permissions).unwrap();
}

/// Whether the read-only flag actually blocks writing (not the case for root)
fn readonly_is_enforced(path: &std::path::Path) -> bool {
    std::fs::OpenOptions::new().write(true).open(path).is_err()
}

#[test]
fn test_save_refuses_read_only_destination() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let dest = dir.path().join("read-only.mp3");
    std::fs::copy(&src, &dest).unwrap();
    let before = std::fs::read(&dest).unwrap();
    set_readonly(&dest, true);
    if !readonly_is_enforced(&dest) {
        set_readonly(&dest, false);
        return;
    }

    let mut t = load_path(&src);
    t.set_title(Either::A("Read only".to_string())).unwrap();
    let result = t.save_sync(Some(Either::B(dest.to_str().unwrap().to_string())));

    let after = std::fs::read(&dest).unwrap();
    let still_readonly = std::fs::metadata(&dest).unwrap().permissions().readonly();
    set_readonly(&dest, false);

    let error = result.err().expect("save should fail");
    assert!(
        error.reason.contains("is not writable"),
        "unexpected error: {}",
        error.reason
    );
    assert_eq!(after, before);
    assert!(still_readonly);
    assert!(temp_entries(dir.path()).is_empty());
}

#[test]
fn test_in_place_save_refuses_read_only_file() {
    let (dir, path) = copy_sample_to_temp("mp3.mp3");
    let before = std::fs::read(&path).unwrap();
    set_readonly(&path, true);
    if !readonly_is_enforced(&path) {
        set_readonly(&path, false);
        return;
    }

    let mut t = load_path(&path);
    t.set_title(Either::A("Read only".to_string())).unwrap();
    let result = t.save_sync(None);

    let after = std::fs::read(&path).unwrap();
    set_readonly(&path, false);

    assert!(result.is_err(), "save should fail");
    assert_eq!(after, before);
    assert!(temp_entries(dir.path()).is_empty());

    // The error names the path as passed to `load`, not the canonical one
    // (`\\?\C:\...` on Windows).
    let error = result.err().unwrap();
    let expected = format!("Destination file '{}' is not writable", path.display());
    assert!(
        error.reason.starts_with(&expected),
        "unexpected error: {}",
        error.reason
    );
    assert!(!error.reason.contains(r"\\?\"));
}

#[test]
fn test_custom_path_save_from_read_only_source() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let existing = dir.path().join("existing.mp3");
    let new = dir.path().join("new.mp3");
    std::fs::copy(&src, &existing).unwrap();
    set_readonly(&src, true);

    let mut t = load_path(&src);
    t.set_title(Either::A("Read-only source".to_string()))
        .unwrap();
    let existing_result = t.save_sync(Some(Either::B(existing.to_str().unwrap().to_string())));
    let new_result = t.save_sync(Some(Either::B(new.to_str().unwrap().to_string())));

    // Restore before asserting, so the temporary directory can be deleted on Windows.
    let new_readonly = std::fs::metadata(&new).map(|m| m.permissions().readonly());
    set_readonly(&src, false);
    if new.exists() {
        set_readonly(&new, false);
    }

    assert!(
        existing_result.is_ok(),
        "save to existing destination failed"
    );
    assert!(new_result.is_ok(), "save to new destination failed");
    assert_eq!(
        load_path(&existing).title().as_deref(),
        Some("Read-only source")
    );
    assert_eq!(load_path(&new).title().as_deref(), Some("Read-only source"));
    // The existing destination keeps its permissions, a new one gets the source's.
    // On Windows, tempfile's `persist` resets file attributes, so only Unix keeps
    // the read-only flag on the new file.
    assert!(!std::fs::metadata(&existing)
        .unwrap()
        .permissions()
        .readonly());
    #[cfg(unix)]
    assert!(new_readonly.unwrap());
    #[cfg(not(unix))]
    let _ = new_readonly;
    assert!(temp_entries(dir.path()).is_empty());
}

#[cfg(unix)]
#[test]
fn test_save_to_dangling_symlink_writes_link_target() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let link = dir.path().join("link.mp3");
    let target = dir.path().join("target.mp3");
    std::os::unix::fs::symlink("target.mp3", &link).unwrap();

    let mut t = load_path(&src);
    t.set_title(Either::A("Dangling".to_string())).unwrap();
    t.save_sync(Some(Either::B(link.to_str().unwrap().to_string())))
        .unwrap();

    assert!(std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(load_path(&target).title().as_deref(), Some("Dangling"));
}

#[cfg(unix)]
#[test]
fn test_save_to_symlink_loop_fails() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let a = dir.path().join("a.mp3");
    let b = dir.path().join("b.mp3");
    std::os::unix::fs::symlink(&b, &a).unwrap();
    std::os::unix::fs::symlink(&a, &b).unwrap();

    let t = load_path(&src);
    let error = t
        .save_sync(Some(Either::B(a.to_str().unwrap().to_string())))
        .err()
        .expect("save should fail");

    assert!(
        error.reason.contains("Too many levels of symbolic links"),
        "unexpected error: {}",
        error.reason
    );
}

#[test]
fn test_buffer_loaded_file_rejects_path_save() {
    let t = music_file_from_buffer("mp3.mp3");

    let error = t.save_sync(None).err().expect("save should fail");
    assert_eq!(error.status, napi::Status::InvalidArg);

    let error = t
        .save_sync(Some(Either::B("out.mp3".to_string())))
        .err()
        .expect("save should fail");
    assert_eq!(error.status, napi::Status::InvalidArg);
}

#[test]
fn test_custom_path_error_names_caller_path() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let dest = dir.path().join("read-only.mp3");
    std::fs::copy(&src, &dest).unwrap();
    set_readonly(&dest, true);
    if !readonly_is_enforced(&dest) {
        set_readonly(&dest, false);
        return;
    }

    let dest_arg = dest.to_str().unwrap().to_string();
    let t = load_path(&src);
    let result = t.save_sync(Some(Either::B(dest_arg.clone())));
    set_readonly(&dest, false);

    let error = result.err().expect("save should fail");
    let expected = format!("Destination file '{dest_arg}' is not writable");
    assert!(
        error.reason.starts_with(&expected),
        "unexpected error: {}",
        error.reason
    );
}

#[cfg(unix)]
#[test]
fn test_new_destination_gets_source_mode() {
    use std::os::unix::fs::PermissionsExt;

    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    std::fs::set_permissions(&src, std::fs::Permissions::from_mode(0o640)).unwrap();
    let dest = dir.path().join("new.mp3");

    let t = load_path(&src);
    t.save_sync(Some(Either::B(dest.to_str().unwrap().to_string())))
        .unwrap();

    let mode = std::fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
}

#[cfg(unix)]
#[test]
fn test_owner_writable_adds_only_owner_write_bit() {
    use std::os::unix::fs::PermissionsExt;

    let permissions = crate::music_file::owner_writable(std::fs::Permissions::from_mode(0o444));
    assert_eq!(permissions.mode() & 0o777, 0o644);
}

#[cfg(not(unix))]
#[test]
fn test_owner_writable_clears_read_only_flag() {
    let (_dir, path) = copy_sample_to_temp("mp3.mp3");
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);

    assert!(!crate::music_file::owner_writable(permissions).readonly());
}

#[cfg(windows)]
#[test]
fn test_display_path_strips_verbatim_prefix() {
    use crate::music_file::display_path;
    use std::path::Path;

    assert_eq!(
        display_path(Path::new(r"\\?\C:\music\song.mp3")),
        r"C:\music\song.mp3"
    );
    assert_eq!(
        display_path(Path::new(r"\\?\UNC\server\share\song.mp3")),
        r"\\server\share\song.mp3"
    );
    assert_eq!(
        display_path(Path::new(r"C:\music\song.mp3")),
        r"C:\music\song.mp3"
    );
}

#[cfg(not(windows))]
#[test]
fn test_display_path_keeps_unix_paths() {
    use crate::music_file::display_path;
    use std::path::Path;

    assert_eq!(
        display_path(Path::new("/music/song.mp3")),
        "/music/song.mp3"
    );
}

#[cfg(unix)]
#[test]
fn test_save_to_dangling_absolute_symlink_writes_link_target() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let link = dir.path().join("link.mp3");
    let target = dir.path().join("target.mp3");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let t = load_path(&src);
    t.save_sync(Some(Either::B(link.to_str().unwrap().to_string())))
        .unwrap();

    assert!(std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(target.is_file());
}

#[cfg(unix)]
#[test]
fn test_save_through_symlink_chain_writes_final_target() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let first = dir.path().join("first.mp3");
    let second = dir.path().join("second.mp3");
    let target = dir.path().join("target.mp3");
    std::os::unix::fs::symlink("second.mp3", &first).unwrap();
    std::os::unix::fs::symlink("target.mp3", &second).unwrap();

    let mut t = load_path(&src);
    t.set_title(Either::A("Chain".to_string())).unwrap();
    t.save_sync(Some(Either::B(first.to_str().unwrap().to_string())))
        .unwrap();

    for link in [&first, &second] {
        assert!(std::fs::symlink_metadata(link)
            .unwrap()
            .file_type()
            .is_symlink());
    }
    assert_eq!(load_path(&target).title().as_deref(), Some("Chain"));
}

#[cfg(unix)]
#[test]
fn test_save_to_symlink_into_missing_directory_fails() {
    let (dir, src) = copy_sample_to_temp("mp3.mp3");
    let link = dir.path().join("link.mp3");
    std::os::unix::fs::symlink("missing/target.mp3", &link).unwrap();

    let t = load_path(&src);
    let error = t
        .save_sync(Some(Either::B(link.to_str().unwrap().to_string())))
        .err()
        .expect("save should fail");

    assert!(
        error.reason.starts_with("Failed accessing"),
        "unexpected error: {}",
        error.reason
    );
    assert!(!dir.path().join("missing").exists());
}

#[test]
fn test_buffer_save_error_has_context() {
    let t = music_file_from_buffer("mp3.mp3");

    let error = t
        .save_sync(Some(Either::A(vec![1u8, 2, 3].into())))
        .err()
        .expect("save should fail");

    assert!(
        error.reason.starts_with("Failed saving to buffer: "),
        "unexpected error: {}",
        error.reason
    );
}

#[test]
fn test_load_errors_are_invalid_arguments() {
    for path in ["", "does-not-exist.mp3"] {
        let error = MusicFile::load_sync(Either::B(path.to_string()))
            .err()
            .expect("load should fail");
        assert_eq!(error.status, napi::Status::InvalidArg, "path {path:?}");
    }
}

#[test]
fn test_load_records_file_size() {
    let (_dir, path) = copy_sample_to_temp("mp3.mp3");
    let expected = std::fs::metadata(&path).unwrap().len();

    let t = load_path(&path);

    match &t.inner {
        crate::music_file::MusicFileInner::Path { file_size, .. } => {
            assert_eq!(*file_size, expected)
        }
        crate::music_file::MusicFileInner::Buffer { .. } => panic!("loaded from a path"),
    }
}

/// Year stored in the trailing ID3v1 tag of `buffer`
fn id3v1_year(buffer: &[u8]) -> String {
    let tag = &buffer[buffer.len() - 128..];
    assert_eq!(&tag[..3], b"TAG");
    String::from_utf8_lossy(&tag[93..97]).into_owned()
}

#[test]
fn test_set_year_updates_id3v1_on_save() {
    let source = id3v1_only_mp3();
    let mut t = MusicFile::load_sync(Either::A(Uint8Array::with_data_copied(&source))).unwrap();

    t.set_year(Either::A(2024.0)).unwrap();

    let saved = save_to_new_buffer(&t, source);
    assert_eq!(id3v1_year(&saved), "2024");
    let reloaded = MusicFile::load_sync(Either::A(saved.into())).unwrap();
    assert_eq!(reloaded.year(), Some(2024));
}
