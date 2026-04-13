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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_picture(data: Vec<u8>, pic_type: PictureType, mime: MimeType, desc: Option<&str>) -> Picture {
        let mut builder = Picture::unchecked(data).pic_type(pic_type).mime_type(mime);
        if let Some(d) = desc {
            builder = builder.description(d.to_string());
        }
        builder.build()
    }

    #[test]
    fn test_from_lofty_picture_basic() {
        let data = vec![1u8, 2, 3, 4, 5];
        let pic = make_picture(data.clone(), PictureType::CoverFront, MimeType::Jpeg, None);

        let meta = from_lofty_picture(&pic);

        assert_eq!(meta.mime_type.as_deref(), Some("image/jpeg"));
        assert_eq!(meta.data.as_ref(), data.as_slice());
        assert_eq!(meta.description, None);
        // Verify the cover_type is the APE key for CoverFront
        assert_eq!(
            meta.cover_type,
            PictureType::CoverFront.as_ape_key().expect("CoverFront should have APE key")
        );
    }

    #[test]
    fn test_from_lofty_picture_with_description() {
        let data = vec![10u8, 20, 30];
        let pic = make_picture(data.clone(), PictureType::CoverFront, MimeType::Png, Some("Album Art"));

        let meta = from_lofty_picture(&pic);

        assert_eq!(meta.mime_type.as_deref(), Some("image/png"));
        assert_eq!(meta.description.as_deref(), Some("Album Art"));
        assert_eq!(meta.data.as_ref(), data.as_slice());
    }

    #[test]
    fn test_from_lofty_picture_various_mime_types() {
        let data = vec![0u8; 16];

        let jpeg = make_picture(data.clone(), PictureType::CoverFront, MimeType::Jpeg, None);
        assert_eq!(from_lofty_picture(&jpeg).mime_type.as_deref(), Some("image/jpeg"));

        let png = make_picture(data.clone(), PictureType::CoverFront, MimeType::Png, None);
        assert_eq!(from_lofty_picture(&png).mime_type.as_deref(), Some("image/png"));

        let gif = make_picture(data.clone(), PictureType::CoverFront, MimeType::Gif, None);
        assert_eq!(from_lofty_picture(&gif).mime_type.as_deref(), Some("image/gif"));
    }

    #[test]
    fn test_from_lofty_picture_slice_empty() {
        let result = from_lofty_picture_slice(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_from_lofty_picture_slice_single() {
        let pic = make_picture(vec![1u8, 2, 3], PictureType::CoverFront, MimeType::Jpeg, None);
        let result = from_lofty_picture_slice(&[pic]);
        assert!(result.is_some());
        let pics = result.unwrap();
        assert_eq!(pics.len(), 1);
        assert_eq!(pics[0].mime_type.as_deref(), Some("image/jpeg"));
    }

    #[test]
    fn test_from_lofty_picture_slice_multiple() {
        let pic1 = make_picture(vec![1u8, 2], PictureType::CoverFront, MimeType::Jpeg, None);
        let pic2 = make_picture(vec![3u8, 4], PictureType::CoverBack, MimeType::Png, Some("Back"));
        let result = from_lofty_picture_slice(&[pic1, pic2]);
        assert!(result.is_some());
        let pics = result.unwrap();
        assert_eq!(pics.len(), 2);
        assert_eq!(pics[0].mime_type.as_deref(), Some("image/jpeg"));
        assert_eq!(pics[1].mime_type.as_deref(), Some("image/png"));
        assert_eq!(pics[1].description.as_deref(), Some("Back"));
    }

    #[test]
    fn test_to_lofty_picture_round_trip() {
        let data = vec![1u8, 2, 3, 4, 5];
        let original = make_picture(data.clone(), PictureType::CoverFront, MimeType::Jpeg, Some("Cover Art"));
        let meta = from_lofty_picture(&original);
        let converted = to_lofty_picture(&meta);

        assert_eq!(converted.data(), data.as_slice());
        assert_eq!(converted.mime_type().map(|m| m.as_str()), Some("image/jpeg"));
        assert_eq!(converted.description(), Some("Cover Art"));
    }

    #[test]
    fn test_to_lofty_picture_no_mime() {
        let meta = MetaPicture {
            cover_type: "Cover".to_string(),
            mime_type: None,
            description: None,
            data: vec![1u8, 2, 3].into(),
        };
        let pic = to_lofty_picture(&meta);
        assert_eq!(pic.data(), &[1u8, 2, 3]);
    }

    #[test]
    fn test_to_lofty_picture_preserves_data() {
        let data: Vec<u8> = (0..=255u8).collect();
        let meta = MetaPicture {
            cover_type: "Cover".to_string(),
            mime_type: Some("image/jpeg".to_string()),
            description: Some("Full range".to_string()),
            data: data.clone().into(),
        };
        let pic = to_lofty_picture(&meta);
        assert_eq!(pic.data(), data.as_slice());
    }
}
