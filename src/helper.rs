use lofty::{
    file::{AudioFile, TaggedFileExt},
    tag::{ItemKey, Tag, TagItem, TagType},
};
use napi::{bindgen_prelude::Null, Either, Error, Result, Status};

use crate::music_file::MusicFile;

/// Public name of a tag type, as exposed by `MusicFile.tagType`
pub(crate) fn tag_type_name(tag_type: TagType) -> Option<&'static str> {
    match tag_type {
        TagType::AiffText => Some("AIFF"),
        TagType::Ape => Some("APE"),
        TagType::Id3v1 => Some("ID3V1"),
        TagType::Id3v2 => Some("ID3V2"),
        TagType::Mp4Ilst => Some("ILST"),
        TagType::RiffInfo => Some("RIFF"),
        TagType::VorbisComments => Some("VORBIS"),
        _ => None,
    }
}

/// Copy items and pictures from `src` into a new tag of another type.
///
/// Items without a mapping in the destination type are skipped. Lyrics are
/// stored under different keys depending on the tag type (ID3v2 only supports
/// `UnsyncLyrics`), so they are moved to the other key when needed.
fn seed_tag(dest: &mut Tag, src: &Tag) {
    for item in src.items() {
        // `push` keeps multi-valued items, unlike `insert` which replaces.
        if dest.push(item.clone()) {
            continue;
        }

        let alternate = match item.key() {
            ItemKey::Lyrics => ItemKey::UnsyncLyrics,
            ItemKey::UnsyncLyrics => ItemKey::Lyrics,
            _ => continue,
        };
        dest.push(TagItem::new(alternate, item.value().clone()));
    }

    for picture in src.pictures() {
        dest.push_picture(picture.clone());
    }
}

/// Insert a text item, failing when the tag type has no mapping for `item_key`.
///
/// lofty's `Tag::insert_text` returns `false` and silently drops the value in
/// that case, which would make a setter look successful while nothing is stored.
pub(crate) fn insert_text_checked(
    tag: &mut Tag,
    item_key: ItemKey,
    field: &str,
    text: String,
) -> Result<()> {
    if tag.insert_text(item_key, text) {
        return Ok(());
    }

    Err(Error::new(
        Status::InvalidArg,
        format!(
            "{field} is not supported by the {} tag",
            tag_type_name(tag.tag_type()).unwrap_or("current")
        ),
    ))
}

impl MusicFile {
    pub(crate) fn validate_integer(value: f64, field: &str, min: u64, max: u64) -> Result<u32> {
        if !value.is_finite() || value.fract() != 0.0 || value < min as f64 || value > max as f64 {
            return Err(Error::new(
                Status::InvalidArg,
                format!("{field} must be a finite integer in [{min}, {max}]"),
            ));
        }

        Ok(value as u32)
    }

    /// Execute a function on the primary or first available tag
    pub(crate) fn tag<R, F>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Tag) -> Option<R>,
    {
        if !self.file.contains_tag() {
            return None;
        }

        self.file
            .primary_tag()
            .or_else(|| self.file.first_tag())
            .and_then(f)
    }

    /// Execute a mutable function on the tag that should receive a write.
    ///
    /// - If the primary tag exists, `f` runs on it.
    /// - If only another tag exists (e.g. ID3v1 or APE on an MP3), `f` runs on that
    ///   tag first. Only when that tag cannot store the field is a primary tag
    ///   created, seeded with the existing tag's items and pictures.
    /// - A new primary tag is inserted only after `f` succeeds on it, so a failed
    ///   setter never changes the file's tags.
    ///
    /// `f` may run twice and must leave the tag unchanged when it returns an error.
    pub(crate) fn tag_mut<R, F>(&mut self, f: F) -> Result<R>
    where
        F: Fn(&mut Tag) -> Result<R>,
    {
        let primary_type = self.file.primary_tag_type();
        if let Some(tag) = self.file.tag_mut(primary_type) {
            return f(tag);
        }

        let existing_error = match self.file.first_tag_mut() {
            Some(tag) => match f(tag) {
                Ok(result) => return Ok(result),
                Err(error) => Some(error),
            },
            None => None,
        };

        let mut candidate = Tag::new(primary_type);
        if let Some(existing) = self.file.first_tag() {
            seed_tag(&mut candidate, existing);
        }

        match f(&mut candidate) {
            Ok(result) => {
                self.file.insert_tag(candidate);
                Ok(result)
            }
            // Report the error for the tag that `tagType` still shows.
            Err(error) => Err(existing_error.unwrap_or(error)),
        }
    }

    pub(crate) fn set_text_field(
        &mut self,
        item_key: ItemKey,
        field: &str,
        value: Either<String, Null>,
    ) -> Result<()> {
        let value = match value {
            Either::A(value) => Some(value),
            Either::B(_) => None,
        };

        self.tag_mut(|tag| match &value {
            Some(value) => insert_text_checked(tag, item_key, field, value.clone()),
            None => {
                tag.remove_key(item_key);
                Ok(())
            }
        })
    }

    pub(crate) fn set_integer_field(
        &mut self,
        item_key: ItemKey,
        field: &str,
        value: Either<f64, Null>,
    ) -> Result<()> {
        let value = match value {
            Either::A(value) => Some(Self::validate_integer(value, field, 0, u32::MAX as u64)?),
            Either::B(_) => None,
        };

        self.tag_mut(|tag| match value {
            Some(value) => insert_text_checked(tag, item_key, field, value.to_string()),
            None => {
                tag.remove_key(item_key);
                Ok(())
            }
        })
    }

    pub(crate) fn set_gain_value<F>(
        &mut self,
        item_key: ItemKey,
        field: &str,
        value: Either<f64, Null>,
        f: F,
    ) -> Result<()>
    where
        F: Fn(f64) -> String,
    {
        let value = match value {
            Either::A(value) => {
                if !value.is_finite() {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "ReplayGain value must be finite",
                    ));
                }
                Some(value)
            }
            Either::B(_) => None,
        };

        self.tag_mut(|tag| match value {
            Some(value) => insert_text_checked(tag, item_key, field, f(value)),
            None => {
                tag.remove_key(item_key);
                Ok(())
            }
        })
    }
}
