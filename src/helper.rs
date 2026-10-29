use lofty::{
    file::{AudioFile, TaggedFileExt},
    tag::{ItemKey, Tag, TagType},
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

    /// Execute a mutable function on the primary tag.
    ///
    /// If the file has no primary tag, one is created first. When the file already
    /// has another tag (e.g. only ID3v1 or APE on an MP3), the new primary tag is
    /// seeded with that tag's items and pictures, so reads stay consistent and
    /// writes are not limited by the weaker tag format.
    pub(crate) fn tag_mut<R, F>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Tag) -> Result<R>,
    {
        let primary_type = self.file.primary_tag_type();

        if self.file.tag(primary_type).is_none() {
            let mut primary = Tag::new(primary_type);
            if let Some(existing) = self.file.first_tag() {
                for item in existing.items() {
                    // Items without a mapping in the primary tag type are skipped.
                    primary.insert(item.clone());
                }
                for picture in existing.pictures() {
                    primary.push_picture(picture.clone());
                }
            }
            self.file.insert_tag(primary);
        }

        match self.file.primary_tag_mut() {
            Some(tag) => f(tag),
            None => match self.file.first_tag_mut() {
                Some(tag) => f(tag),
                None => Err(Error::from_reason(
                    "UNREACHABLE: a tag must be available after inserting",
                )),
            },
        }
    }

    pub(crate) fn set_text_field(
        &mut self,
        item_key: ItemKey,
        field: &str,
        value: Either<String, Null>,
    ) -> Result<()> {
        self.tag_mut(|tag| match value {
            Either::A(v) => insert_text_checked(tag, item_key, field, v),
            Either::B(_) => {
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
        F: FnOnce(f64) -> String,
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
