use lofty::picture::{MimeType, Picture, PictureType};
use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;

#[napi]
pub struct MetaPicture {
    #[napi(
        ts_type = r#""Cover Art (Other)" | "Cover Art (Png Icon)" | "Cover Art (Icon)" | "Cover Art (Front)" | "Cover Art (Back)" | "Cover Art (Leaflet)" | "Cover Art (Media)" | "Cover Art (Lead Artist)" | "Cover Art (Artist)" | "Cover Art (Conductor)" | "Cover Art (Band)" | "Cover Art (Composer)" | "Cover Art (Lyricist)" | "Cover Art (Recording Location)" | "Cover Art (During Recording)" | "Cover Art (During Performance)" | "Cover Art (Video Capture)" | "Cover Art (Fish)" | "Cover Art (Illustration)" | "Cover Art (Band Logotype)" | "Cover Art (Publisher Logotype)" | "Unknown""#
    )]
    pub cover_type: String,
    pub mime_type: Option<String>,
    pub description: Option<String>,
    pub data: Uint8Array,
    /// Raw type byte of a picture read with a non-standard type, so it survives a
    /// read/write round trip while `cover_type` is still "Unknown".
    pub(crate) undefined_type: Option<u8>,
}

#[napi]
impl MetaPicture {
    #[napi(constructor)]
    pub fn new(mime: String, data: Uint8Array, desc: Option<String>) -> Self {
        MetaPicture {
            cover_type: PictureType::CoverFront.as_ape_key().unwrap().to_owned(),
            mime_type: Some(mime),
            description: desc,
            data,
            undefined_type: None,
        }
    }
}

pub fn from_lofty_picture_slice(pics: &[Picture]) -> Option<Vec<MetaPicture>> {
    if pics.is_empty() {
        return None;
    }

    let mut result = Vec::with_capacity(pics.len());
    result.extend(pics.iter().map(from_lofty_picture));
    Some(result)
}

pub fn from_lofty_picture(pic: &Picture) -> MetaPicture {
    MetaPicture {
        cover_type: pic.pic_type().as_ape_key().unwrap_or("Unknown").to_owned(),
        mime_type: pic.mime_type().map(|mime| mime.as_str().to_owned()),
        description: pic.description().map(ToOwned::to_owned),
        data: pic.data().into(),
        undefined_type: match pic.pic_type() {
            PictureType::Undefined(raw) => Some(raw),
            _ => None,
        },
    }
}

pub fn to_lofty_picture(pic: &MetaPicture) -> Picture {
    let pic_type = match (
        PictureType::from_ape_key(pic.cover_type.as_str()),
        pic.undefined_type,
    ) {
        (PictureType::Undefined(_), Some(raw)) => PictureType::Undefined(raw),
        (pic_type, _) => pic_type,
    };
    let mut pic_builder = Picture::unchecked(pic.data.to_vec()).pic_type(pic_type);

    if let Some(mime_str) = pic.mime_type.as_deref() {
        pic_builder = pic_builder.mime_type(MimeType::from_str(mime_str));
    }

    if let Some(desc) = pic.description.as_ref() {
        pic_builder = pic_builder.description(desc.clone());
    }

    pic_builder.build()
}
