use lofty::{
    file::{AudioFile, TaggedFileExt},
    tag::{ItemKey, Tag},
};
use napi::{bindgen_prelude::Null, Either, Error, Result};

use crate::tagged_file::TaggedFile;

impl TaggedFile {
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

    /// Execute a mutable function on the primary or first available tag
    pub(crate) fn tag_mut<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Tag),
    {
        if !self.file.contains_tag() {
            // If no tag is available in the file, insert an empty one.
            self.file
                .insert_tag(Tag::new(self.file.file_type().primary_tag_type()));
        }

        if let Some(tag) = self.file.primary_tag_mut() {
            f(tag);
            Ok(())
        } else if let Some(tag) = self.file.first_tag_mut() {
            f(tag);
            Ok(())
        } else {
            Err(Error::from_reason(
                "UNREACHABLE: a tag must be available after inserting",
            ))
        }
    }

    pub(crate) fn set_text_field(
        &mut self,
        item_key: ItemKey,
        value: Either<String, Null>,
    ) -> Result<()> {
        self.tag_mut(|tag| match value {
            Either::A(v) => {
                tag.insert_text(item_key, v);
            }
            Either::B(_) => tag.remove_key(item_key),
        })
    }
    pub(crate) fn set_gain_value<F>(
        &mut self,
        item_key: ItemKey,
        value: Either<f64, Null>,
        f: F,
    ) -> Result<()>
    where
        F: FnOnce(f64) -> String,
    {
        self.tag_mut(|tag| match value {
            Either::A(v) => {
                tag.insert_text(item_key, f(v));
            }
            Either::B(_) => tag.remove_key(item_key),
        })
    }
}
