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

    /// Execute a mutable function on the primary or first available tag.
    ///
    /// Without any tag, `f` runs on a new primary tag that is inserted only after
    /// `f` succeeds, so a failed setter never changes the file's tags.
    ///
    /// `f` must leave the tag unchanged when it returns an error.
    pub(crate) fn tag_mut<R, F>(&mut self, f: F) -> Result<R>
    where
        F: Fn(&mut Tag) -> Result<R>,
    {
        if let Some(tag) = self.file.primary_tag_mut() {
            return f(tag);
        }
        if let Some(tag) = self.file.first_tag_mut() {
            return f(tag);
        }

        let mut candidate = Tag::new(self.file.primary_tag_type());
        let result = f(&mut candidate)?;
        self.file.insert_tag(candidate);
        Ok(result)
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
