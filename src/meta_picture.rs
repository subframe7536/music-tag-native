use lofty::picture::{MimeType, Picture, PictureType};
use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;
use std::borrow::Cow;

#[napi]
pub struct MetaPicture {
    #[napi(js_name = "coverType")]
    pub cover_type: String,
    #[napi(js_name = "mimeType")]
    pub mime_type: Option<String>,
    pub description: Option<String>,
    pub data: Uint8Array,
}

#[napi]
impl MetaPicture {
    #[napi(constructor)]
    pub fn new(mime: String, data: Uint8Array, desc: Option<String>) -> Self {
        MetaPicture {
            cover_type: PictureType::CoverFront.as_ape_key().expect("").to_string(),
            mime_type: Some(mime.to_string()),
            description: desc,
            data,
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
        cover_type: pic
            .pic_type()
            .as_ape_key()
            .map_or_else(|| Cow::Borrowed("Unknown"), Cow::Borrowed)
            .into_owned(),
        mime_type: pic.mime_type().map(|mime| mime.as_str().to_string()),
        description: pic.description().map(ToString::to_string),
        data: pic.data().into(),
    }
}

pub fn to_lofty_picture(pic: &MetaPicture) -> Picture {
    let mut pic_builder = Picture::unchecked(pic.data.to_vec())
        .pic_type(PictureType::from_ape_key(&pic.cover_type.as_str()));

    if let Some(mime_str) = pic.mime_type.as_deref() {
        pic_builder = pic_builder.mime_type(MimeType::from_str(mime_str));
    }

    if let Some(desc) = pic.description.as_ref() {
        pic_builder = pic_builder.description(desc.clone());
    }

    pic_builder.build()
}
