use napi::bindgen_prelude::*;
use std::io::Cursor;

use lofty::{
    config::WriteOptions,
    file::{AudioFile, FileType, TaggedFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, ItemKey, Tag, TagType},
};
use napi_derive::napi;

use crate::{
    meta_picture::{from_lofty_picture_slice, to_lofty_picture, MetaPicture},
    utils::{format_replaygain_gain, format_replaygain_peak, parse_replaygain_value},
};

#[napi(custom_finalize)]
pub struct MusicTagger {
    inner: Option<MetaFileInner>,
}

struct MetaFileInner {
    buffer: Vec<u8>,
    file: TaggedFile,
    path: Option<String>,
}

#[napi]
pub enum Quality {
    HQ,
    SQ,
    HiRes,
}

const ERR_NO_TAG: &str = "File must contain at least one tag";
const ERR_DISPOSED: &str = "File has been disposed";
const ERR_NO_BUFFER: &str = "File has no buffer";
const ERR_INVALID_IN_WASM: &str = "This method is invalid in wasm build";
const MIN_BITRATE: u32 = 8;
const MAX_BITRATE: u32 = 10_000;
const HIRES_MIN_SAMPLE_RATE: u32 = 44_100;
const HIRES_MIN_BIT_DEPTH: u8 = 16;

impl MusicTagger {
    #[inline]
    fn inner(&self) -> Result<&MetaFileInner> {
        self.inner
            .as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, ERR_DISPOSED))
    }

    #[inline]
    fn inner_mut(&mut self) -> Result<&mut MetaFileInner> {
        self.inner
            .as_mut()
            .ok_or_else(|| Error::new(Status::GenericFailure, ERR_DISPOSED))
    }

    fn try_tag<R, F>(&self, f: F) -> Result<Option<R>>
    where
        F: FnOnce(&Tag) -> Option<R>,
    {
        let inner = self.inner()?;
        Ok(inner
            .file
            .primary_tag()
            .or_else(|| inner.file.first_tag())
            .and_then(f))
    }

    fn try_tag_mut<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Tag),
    {
        let inner = self.inner_mut()?;

        if let Some(tag) = inner.file.primary_tag_mut() {
            f(tag);
            Ok(())
        } else if let Some(tag) = inner.file.first_tag_mut() {
            f(tag);
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, ERR_NO_TAG))
        }
    }
}

#[napi]
impl MusicTagger {
    #[napi(constructor)]
    pub fn new() -> Self {
        MusicTagger { inner: None }
    }

    /// Load music file from buffer
    #[napi]
    pub fn load_buffer(&mut self, buffer: Uint8Array) -> Result<()> {
        self.dispose();

        let buffer_vec = buffer.to_vec();
        let file = Probe::new(Cursor::new(&buffer_vec))
            .guess_file_type()
            .map_err(|e| Error::new(Status::InvalidArg, &e.to_string()))?
            .read()
            .map_err(|e| Error::new(Status::InvalidArg, &e.to_string()))?;

        // Ensure there's at least one tag
        if file.primary_tag().is_none() && file.first_tag().is_none() {
            Err(Error::new(Status::InvalidArg, ERR_NO_TAG))
        } else {
            self.inner = Some(MetaFileInner {
                buffer: buffer_vec,
                file,
                path: None,
            });
            Ok(())
        }
    }

    /// Load music file from file path
    ///
    /// Invalid in wasm
    #[napi]
    pub fn load_path(&mut self, path: String) -> Result<()> {
        if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
            return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
        }

        self.dispose();
        let file = Probe::open(&path)
            .map_err(|e| Error::new(Status::InvalidArg, &e.to_string()))?
            .guess_file_type()
            .map_err(|e| Error::new(Status::InvalidArg, &e.to_string()))?
            .read()
            .map_err(|e| Error::new(Status::InvalidArg, &e.to_string()))?;

        // Ensure there's at least one tag
        if file.primary_tag().is_none() && file.first_tag().is_none() {
            Err(Error::new(Status::InvalidArg, ERR_NO_TAG))
        } else {
            self.inner = Some(MetaFileInner {
                buffer: Vec::with_capacity(0),
                file,
                path: Some(path),
            });
            Ok(())
        }
    }

    /// Drop current file
    #[napi]
    pub fn dispose(&mut self) {
        if self.inner.is_some() {
            self.inner = None;
        }
    }

    /// Check if current file is disposed
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.inner.is_none()
    }

    /// Save changes to buffer
    #[napi]
    pub fn save_buffer(&mut self) -> Result<()> {
        let inner = self.inner_mut()?;
        if inner.buffer.len() == 0 {
            return Err(Error::new(Status::GenericFailure, ERR_NO_BUFFER));
        }
        let mut cursor = Cursor::new(&mut inner.buffer);
        inner
            .file
            .save_to(&mut cursor, WriteOptions::default())
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to save buffer: {}", e),
                )
            })?;

        Ok(())
    }

    /// Save changes to file path
    ///
    /// Invalid in wasm
    #[napi]
    pub fn save_path(&mut self, path: Option<String>) -> Result<()> {
        if cfg!(all(target_arch = "wasm32", target_os = "wasi")) {
            return Err(Error::new(Status::GenericFailure, ERR_INVALID_IN_WASM));
        }

        let inner = self.inner_mut()?;

        let target_path: &str = match (&path, &inner.path) {
            (Some(p), _) => p.as_str(),
            (None, Some(p)) => p,
            (None, None) => {
                return Err(Error::new(Status::GenericFailure, ERR_NO_BUFFER));
            }
        };

        inner
            .file
            .save_to_path(target_path, WriteOptions::default())
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to save to path '{}': {}", target_path, e),
                )
            })?;

        Ok(())
    }

    /// Get buffer
    /// Make sure run `save()` before
    #[napi(getter)]
    pub fn buffer(&self) -> Result<Uint8Array> {
        Ok(Uint8Array::new(self.inner()?.buffer.clone()))
    }

    /// Audio file quality
    #[napi(getter)]
    pub fn quality(&self) -> Result<Quality> {
        let is_lossless = matches!(
            self.inner()?.file.file_type(),
            FileType::Flac | FileType::Ape | FileType::Aiff | FileType::Wav | FileType::WavPack
        );

        if !is_lossless {
            Ok(Quality::HQ)
        } else {
            match (self.sample_rate()?, self.bit_depth()?) {
                (Some(sr), Some(bd)) if sr > HIRES_MIN_SAMPLE_RATE && bd >= HIRES_MIN_BIT_DEPTH => {
                    Ok(Quality::HiRes)
                }
                _ => Ok(Quality::SQ),
            }
        }
    }

    /// Audio bit depth in bits
    #[napi(getter)]
    pub fn bit_depth(&self) -> Result<Option<u8>> {
        Ok(self.inner()?.file.properties().bit_depth())
    }

    /// Audio bit rate in kbps
    ///
    /// Note: If the audio property not available,
    /// the getter will try to calculates an approximate bitrate including metadata size.
    #[napi(getter)]
    pub fn bit_rate(&self) -> Result<Option<u32>> {
        if let Some(bitrate) = self.inner()?.file.properties().audio_bitrate() {
            return Ok(Some(bitrate));
        }

        let duration = self.inner()?.file.properties().duration();
        if duration.is_zero() {
            return Ok(None);
        }

        let duration_secs = duration.as_secs_f64();
        if duration_secs <= f64::EPSILON {
            return Ok(None);
        }

        let file_size_bytes = self.inner()?.buffer.len() as f64;
        let bitrate_kbps = ((file_size_bytes * 8.0) / (duration_secs * 1000.0)).round() as u32;

        Ok((MIN_BITRATE..=MAX_BITRATE)
            .contains(&bitrate_kbps)
            .then_some(bitrate_kbps))
    }

    /// Audio sample rate in Hz
    #[napi(getter)]
    pub fn sample_rate(&self) -> Result<Option<u32>> {
        Ok(self.inner()?.file.properties().sample_rate())
    }

    /// Number of channels
    #[napi(getter)]
    pub fn channels(&self) -> Result<Option<u8>> {
        Ok(self.inner()?.file.properties().channels())
    }

    /// Duration in milliseconds
    #[napi(getter)]
    pub fn duration(&self) -> Result<u32> {
        Ok(self.inner()?.file.properties().duration().as_millis() as u32)
    }

    /// File's tag type
    #[napi(getter)]
    pub fn tag_type(&self) -> Result<Option<String>> {
        self.try_tag(|tag| match tag.tag_type() {
            TagType::AiffText => Some("AIFF".to_string()),
            TagType::Ape => Some("APE".to_string()),
            TagType::Id3v1 => Some("ID3V1".to_string()),
            TagType::Id3v2 => Some("ID3V2".to_string()),
            TagType::Mp4Ilst => Some("ILST".to_string()),
            TagType::RiffInfo => Some("RIFF".to_string()),
            TagType::VorbisComments => Some("VORBIS".to_string()),
            _ => None,
        })
    }

    /// The title of the music file
    #[napi(getter)]
    pub fn title(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.title().map(String::from))
    }

    #[napi(setter)]
    pub fn set_title(&mut self, title: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_title(title))
    }

    /// The artist of the music file
    #[napi(getter)]
    pub fn artist(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.artist().map(String::from))
    }

    #[napi(setter)]
    pub fn set_artist(&mut self, artist: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_artist(artist))
    }

    /// The album of the music file
    #[napi(getter)]
    pub fn album(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.album().map(String::from))
    }

    #[napi(setter)]
    pub fn set_album(&mut self, album: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_album(album))
    }

    /// The year of the music file
    #[napi(getter)]
    pub fn year(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.year())
    }

    #[napi(setter)]
    pub fn set_year(&mut self, year: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_year(year))
    }

    /// The genre of the music file
    #[napi(getter)]
    pub fn genre(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.genre().map(String::from))
    }

    #[napi(setter)]
    pub fn set_genre(&mut self, genre: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_genre(genre))
    }

    /// The track number of the music file
    #[napi(getter)]
    pub fn track_number(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.track())
    }

    #[napi(setter)]
    pub fn set_track_number(&mut self, track_number: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_track(track_number))
    }

    /// The disc number of the music file
    #[napi(getter)]
    pub fn disc_number(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.disk())
    }

    #[napi(setter)]
    pub fn set_disc_number(&mut self, disc_number: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_disk(disc_number))
    }

    /// The total number of tracks
    #[napi(getter)]
    pub fn track_total(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.track_total())
    }

    #[napi(setter)]
    pub fn set_track_total(&mut self, track_total: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_track_total(track_total))
    }

    /// The total number of discs
    #[napi(getter)]
    pub fn discs_total(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.disk_total())
    }

    #[napi(setter)]
    pub fn set_discs_total(&mut self, discs_total: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_disk_total(discs_total))
    }

    /// The comment of the music file
    #[napi(getter)]
    pub fn comment(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.comment().map(String::from))
    }

    #[napi(setter)]
    pub fn set_comment(&mut self, comment: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_comment(comment))
    }

    /// The Album Artist of the music file
    #[napi(getter)]
    pub fn album_artist(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::AlbumArtist).map(String::from))
    }

    #[napi(setter)]
    pub fn set_album_artist(&mut self, album_artist: String) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(ItemKey::AlbumArtist, album_artist);
        })
    }

    /// The Composer of the music file
    #[napi(getter)]
    pub fn composer(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Composer).map(String::from))
    }

    #[napi(setter)]
    pub fn set_composer(&mut self, composer: String) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(ItemKey::Composer, composer);
        })
    }

    /// The Conductor of the music file
    #[napi(getter)]
    pub fn conductor(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Conductor).map(String::from))
    }

    #[napi(setter)]
    pub fn set_conductor(&mut self, conductor: String) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(ItemKey::Conductor, conductor);
        })
    }

    /// The Lyricist of the music file
    #[napi(getter)]
    pub fn lyricist(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Lyricist).map(String::from))
    }

    #[napi(setter)]
    pub fn set_lyricist(&mut self, lyricist: String) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(ItemKey::Lyricist, lyricist);
        })
    }

    /// The Publisher of the music file
    #[napi(getter)]
    pub fn publisher(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Publisher).map(String::from))
    }

    #[napi(setter)]
    pub fn set_publisher(&mut self, publisher: String) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(ItemKey::Publisher, publisher);
        })
    }

    /// The lyrics of the music file
    #[napi(getter)]
    pub fn lyrics(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Lyrics).map(String::from))
    }

    #[napi(setter)]
    pub fn set_lyrics(&mut self, lyrics: String) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(ItemKey::Lyrics, lyrics);
        })
    }

    /// The copyright of the music file
    #[napi(getter)]
    pub fn copyright(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::CopyrightMessage).map(String::from))
    }

    #[napi(setter)]
    pub fn set_copyright(&mut self, copyright: String) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(ItemKey::CopyrightMessage, copyright);
        })
    }

    /// The track replay gain of the music file
    #[napi(getter)]
    pub fn track_replay_gain(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainTrackGain)?))
    }

    #[napi(setter)]
    pub fn set_track_replay_gain(&mut self, track_replay_gain: f64) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(
                ItemKey::ReplayGainTrackGain,
                format_replaygain_gain(track_replay_gain),
            );
        })
    }

    /// The track replay peak of the music file
    #[napi(getter)]
    pub fn track_replay_peak(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainTrackPeak)?))
    }

    #[napi(setter)]
    pub fn set_track_replay_peak(&mut self, track_replay_peak: f64) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(
                ItemKey::ReplayGainTrackPeak,
                format_replaygain_peak(track_replay_peak),
            );
        })
    }

    /// The album replay gain of the music file
    #[napi(getter)]
    pub fn album_replay_gain(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainAlbumGain)?))
    }

    #[napi(setter)]
    pub fn set_album_replay_gain(&mut self, album_replay_gain: f64) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(
                ItemKey::ReplayGainAlbumGain,
                format_replaygain_gain(album_replay_gain),
            );
        })
    }

    /// The album replay peak of the music file
    #[napi(getter)]
    pub fn album_replay_peak(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainAlbumPeak)?))
    }

    #[napi(setter)]
    pub fn set_album_replay_peak(&mut self, album_replay_peak: f64) -> Result<()> {
        self.try_tag_mut(|tag| {
            tag.insert_text(
                ItemKey::ReplayGainAlbumPeak,
                format_replaygain_peak(album_replay_peak),
            );
        })
    }

    /// The pictures of the music file
    #[napi(getter)]
    pub fn get_pictures(&self) -> Result<Option<Vec<MetaPicture>>> {
        self.try_tag(|tag| from_lofty_picture_slice(tag.pictures()))
    }

    #[napi(setter)]
    pub fn set_pictures(&mut self, pictures: Vec<&MetaPicture>) -> Result<()> {
        self.try_tag_mut(|tag| {
            for (i, picture) in pictures.into_iter().enumerate() {
                tag.set_picture(i, to_lofty_picture(&picture));
            }
        })
    }
}

impl ObjectFinalize for MusicTagger {
    fn finalize(self, env: Env) -> Result<()> {
        if let Some(inner) = self.inner {
            let len = inner.buffer.len();
            env.adjust_external_memory(-(len as i64))?;
        }
        Ok(())
    }
}
