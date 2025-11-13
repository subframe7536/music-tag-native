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

const ERR_NO_TAG: &str = "File must contain at least one tag";
const ERR_DISPOSED: &str = "File has been disposed";
const ERR_INVALID_IN_WASM: &str = "This method is invalid in wasm build";

impl MusicTagger {
    /// Get a reference to the internal file state
    #[inline]
    fn inner(&self) -> Result<&MetaFileInner> {
        self.inner
            .as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, ERR_DISPOSED))
    }

    /// Get a mutable reference to the internal file state
    #[inline]
    fn inner_mut(&mut self) -> Result<&mut MetaFileInner> {
        self.inner
            .as_mut()
            .ok_or_else(|| Error::new(Status::GenericFailure, ERR_DISPOSED))
    }

    /// Execute a function on the primary or first available tag
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

    /// Execute a mutable function on the primary or first available tag
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

    fn set_text_field(&mut self, item_key: ItemKey, value: Either<String, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match value {
            Either::A(v) => {
                tag.insert_text(item_key, v);
            }
            Either::B(_) => tag.remove_key(&item_key),
        })
    }
    fn set_gain_value<F>(&mut self, item_key: ItemKey, value: Either<f64, Null>, f: F) -> Result<()>
    where
        F: FnOnce(f64) -> String,
    {
        self.try_tag_mut(|tag| match value {
            Either::A(v) => {
                tag.insert_text(item_key, f(v));
            }
            Either::B(_) => tag.remove_key(&item_key),
        })
    }
}

#[napi]
impl MusicTagger {
    /// @constructor
    /// Creates a new MusicTagger instance
    ///
    /// The instance starts in an uninitialized state. You must call either
    /// `loadBuffer()` (Browser) or `loadPath()` (Node.js) to load an audio
    /// file before using other methods.
    #[napi(constructor)]
    pub fn new() -> Self {
        MusicTagger { inner: None }
    }

    /// Load music file from a byte buffer (dispose the old one before)
    ///
    /// @param buffer A Uint8Array containing the audio file data
    ///
    /// @throws If the buffer doesn't contain a valid audio file
    /// @throws If the file doesn't contain any metadata tags
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

    /// Load music file from a file path (dispose the old one before)
    ///
    /// @param path The file system path to the audio file
    ///
    /// @throws If the path doesn't exist or isn't accessible
    /// @throws If the file doesn't contain a valid audio format
    /// @throws If the file doesn't contain any metadata tags
    /// @throws If runs in WebAssembly environments (due to file system restrictions).
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

    /// Dispose of the currently loaded file
    ///
    /// Releases all resources associated with the loaded audio file.
    /// After calling this method, the instance returns to an uninitialized state.
    ///
    /// @note Any unsaved changes will be lost when disposing.
    #[napi]
    pub fn dispose(&mut self) {
        if self.inner.is_some() {
            self.inner = None;
        }
    }

    /// Check if the current file is disposed. Use this method to verify
    /// if the instance is ready for operations before calling other methods.
    ///
    /// @returns `true` if no file is currently loaded, `false` otherwise
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.inner.is_none()
    }

    /// Save metadata changes back to the internal buffer
    ///
    /// @throws If no file or buffer loaded
    /// @throws If saving fails due to file format constraints
    #[napi]
    pub fn save(&mut self) -> Result<()> {
        let inner = self.inner_mut()?;
        if inner.buffer.len() > 0 {
            let mut cursor = Cursor::new(&mut inner.buffer);
            inner
                .file
                .save_to(&mut cursor, WriteOptions::default())
                .map_err(|e| {
                    Error::new(
                        Status::GenericFailure,
                        format!("Failed to save buffer: {}", e),
                    )
                })
        } else if inner.path.is_some() {
            let p = inner.path.as_ref().unwrap();
            inner
                .file
                .save_to_path(p, WriteOptions::default())
                .map_err(|e| {
                    Error::new(
                        Status::GenericFailure,
                        format!("Failed saving to existing file '{}': {}", p, e),
                    )
                })
        } else {
            Err(Error::new(
                Status::GenericFailure,
                "No file path or buffer setup",
            ))
        }
    }

    /// Current audio file buffer as a `Uint8Array`
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note For files loaded via `loadBuffer()`, call `saveBuffer()` first to ensure
    /// metadata changes are applied. For files loaded via `loadPath()`, this
    /// returns an empty buffer.
    #[napi(getter)]
    pub fn buffer(&self) -> Result<Uint8Array> {
        Ok(Uint8Array::new(self.inner()?.buffer.clone()))
    }

    /// Audio quality classification ("HQ", "SQ", or "HiRes")
    ///
    /// @throws If no file or buffer loaded
    ///
    /// Quality is determined based on file format, sample rate, and bit depth:
    /// - HQ: Lossy formats (MP3, AAC, etc.)
    /// - SQ: Lossless formats at CD quality (44.1kHz, 16-bit)
    /// - HiRes: Lossless formats exceeding CD quality (>44.1kHz, >=16-bit)
    #[napi(getter)]
    pub fn quality(&self) -> Result<String> {
        let is_lossless = matches!(
            self.inner()?.file.file_type(),
            FileType::Flac | FileType::Ape | FileType::Aiff | FileType::Wav | FileType::WavPack
        );

        if !is_lossless {
            Ok(String::from("HQ"))
        } else {
            match (self.sample_rate()?, self.bit_depth()?) {
                (Some(sr), Some(bd)) if sr > 44100 && bd >= 16 => Ok(String::from("HiRes")),
                _ => Ok(String::from("SQ")),
            }
        }
    }

    /// Audio bit depth in bits, or `null` if not available
    ///
    /// @throws If no file or buffer loaded
    ///
    /// Common values: 16 (CD quality), 24 (Hi-Res), 32 (studio quality)
    #[napi(getter)]
    pub fn bit_depth(&self) -> Result<Option<u8>> {
        Ok(self.inner()?.file.properties().bit_depth())
    }

    /// Audio bit rate in kbps, or `null` if not available
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note If the audio properties don't provide a bitrate, this method calculates
    /// an approximate bitrate based on file size and duration, including metadata.
    /// The calculated bitrate is constrained between MIN_BITRATE and MAX_BITRATE.
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

        Ok((8..=100_00).contains(&bitrate_kbps).then_some(bitrate_kbps))
    }

    /// Audio sample rate in Hz, or `null` if not available
    ///
    /// @throws If no file or buffer loaded
    ///
    /// Common values: 44100 (CD), 48000 (DVD), 96000, 192000 (Hi-Res)
    #[napi(getter)]
    pub fn sample_rate(&self) -> Result<Option<u32>> {
        Ok(self.inner()?.file.properties().sample_rate())
    }

    /// Number of audio channels, or `null` if not available
    ///
    /// @throws If no file or buffer loaded
    ///
    /// Common values: 1 (mono), 2 (stereo), 6 (5.1 surround)
    /// Common values: 1 (mono), 2 (stereo), 6 (5.1 surround), 8 (7.1 surround)
    #[napi(getter)]
    pub fn channels(&self) -> Result<Option<u8>> {
        Ok(self.inner()?.file.properties().channels())
    }

    /// Audio duration in seconds, or `null` if not available
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn duration(&self) -> Result<u32> {
        Ok(self.inner()?.file.properties().duration().as_millis() as u32)
    }

    /// File's metadata tag type, or `null` if not recognized
    ///
    /// Supported tag types: "ID3V1", "ID3V2", "APE", "VORBIS", "MP4", "AIFF", "RIFF"
    ///
    /// @throws If no file or buffer loaded
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

    /// Title, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn title(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.title().map(String::from))
    }

    #[napi(setter)]
    pub fn set_title(&mut self, title: Either<String, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match title {
            Either::A(t) => tag.set_title(t),
            _ => tag.remove_title(),
        })
    }

    /// Artist, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn artist(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.artist().map(String::from))
    }

    #[napi(setter)]
    pub fn set_artist(&mut self, artist: Either<String, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match artist {
            Either::A(a) => tag.set_artist(a),
            _ => tag.remove_artist(),
        })
    }

    /// Album, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn album(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.album().map(String::from))
    }

    #[napi(setter)]
    pub fn set_album(&mut self, album: Either<String, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match album {
            Either::A(a) => tag.set_album(a),
            _ => tag.remove_album(),
        })
    }

    /// Year, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn year(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.year())
    }

    #[napi(setter)]
    pub fn set_year(&mut self, year: Either<u32, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match year {
            Either::A(y) => tag.set_year(y),
            _ => tag.remove_year(),
        })
    }

    /// Genre, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn genre(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.genre().map(String::from))
    }

    #[napi(setter)]
    pub fn set_genre(&mut self, genre: Either<String, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match genre {
            Either::A(g) => tag.set_genre(g),
            _ => tag.remove_genre(),
        })
    }

    /// Track number, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn track_number(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.track())
    }

    #[napi(setter)]
    pub fn set_track_number(&mut self, track_number: Either<u32, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match track_number {
            Either::A(t) => tag.set_track(t),
            _ => tag.remove_track(),
        })
    }

    /// Disc number, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn disc_number(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.disk())
    }

    #[napi(setter)]
    pub fn set_disc_number(&mut self, disc_number: Either<u32, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match disc_number {
            Either::A(d) => tag.set_disk(d),
            _ => tag.remove_disk(),
        })
    }

    /// Total number of tracks in the album, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn track_total(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.track_total())
    }

    #[napi(setter)]
    pub fn set_track_total(&mut self, track_total: Either<u32, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match track_total {
            Either::A(t) => tag.set_track_total(t),
            _ => tag.remove_track_total(),
        })
    }

    /// Total number of discs in the album, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn discs_total(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.disk_total())
    }

    #[napi(setter)]
    pub fn set_discs_total(&mut self, discs_total: Either<u32, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match discs_total {
            Either::A(d) => tag.set_disk_total(d),
            _ => tag.remove_disk_total(),
        })
    }

    /// Comment, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn comment(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.comment().map(String::from))
    }

    #[napi(setter)]
    pub fn set_comment(&mut self, comment: Either<String, Null>) -> Result<()> {
        self.try_tag_mut(|tag| match comment {
            Either::A(c) => tag.set_comment(c),
            _ => tag.remove_comment(),
        })
    }

    /// Album artist, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note Album artist differs from track artist and represents the primary artist for the entire album.
    #[napi(getter)]
    pub fn album_artist(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::AlbumArtist).map(String::from))
    }

    #[napi(setter)]
    pub fn set_album_artist(&mut self, album_artist: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::AlbumArtist, album_artist)
    }

    /// Composer, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn composer(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Composer).map(String::from))
    }

    #[napi(setter)]
    pub fn set_composer(&mut self, composer: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Composer, composer)
    }

    /// Conductor, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn conductor(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Conductor).map(String::from))
    }

    #[napi(setter)]
    pub fn set_conductor(&mut self, conductor: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Conductor, conductor)
    }

    /// Lyricist, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn lyricist(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Lyricist).map(String::from))
    }

    #[napi(setter)]
    pub fn set_lyricist(&mut self, lyricist: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Lyricist, lyricist)
    }

    /// Publisher, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn publisher(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Publisher).map(String::from))
    }

    #[napi(setter)]
    pub fn set_publisher(&mut self, publisher: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Publisher, publisher)
    }

    /// Lyrics, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn lyrics(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::Lyrics).map(String::from))
    }

    #[napi(setter)]
    pub fn set_lyrics(&mut self, lyrics: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Lyrics, lyrics)
    }

    /// Copyright information, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    #[napi(getter)]
    pub fn copyright(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.get_string(&ItemKey::CopyrightMessage).map(String::from))
    }

    #[napi(setter)]
    pub fn set_copyright(&mut self, copyright: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::CopyrightMessage, copyright)
    }

    /// Track replay gain in dB, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note Replay gain is used to normalize playback volume across different tracks.
    #[napi(getter)]
    pub fn track_replay_gain(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainTrackGain)?))
    }

    #[napi(setter)]
    pub fn set_track_replay_gain(&mut self, track_replay_gain: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainTrackGain,
            track_replay_gain,
            format_replaygain_gain,
        )
    }

    /// Track replay peak value, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note Replay peak represents the maximum amplitude level in the track.
    #[napi(getter)]
    pub fn track_replay_peak(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainTrackPeak)?))
    }

    #[napi(setter)]
    pub fn set_track_replay_peak(&mut self, track_replay_peak: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainTrackPeak,
            track_replay_peak,
            format_replaygain_peak,
        )
    }

    /// Album replay gain in dB, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note Album replay gain normalizes playback volume across different albums.
    #[napi(getter)]
    pub fn album_replay_gain(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainAlbumGain)?))
    }

    #[napi(setter)]
    pub fn set_album_replay_gain(&mut self, album_replay_gain: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainAlbumGain,
            album_replay_gain,
            format_replaygain_gain,
        )
    }

    /// Album replay peak value, or `null` if not set
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note Album replay peak represents the maximum amplitude level in the album.
    #[napi(getter)]
    pub fn album_replay_peak(&self) -> Result<Option<f64>> {
        self.try_tag(|tag| parse_replaygain_value(tag.get_string(&ItemKey::ReplayGainAlbumPeak)?))
    }

    #[napi(setter)]
    pub fn set_album_replay_peak(&mut self, album_replay_peak: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainAlbumPeak,
            album_replay_peak,
            format_replaygain_peak,
        )
    }

    /// Embedded pictures/album art list, or `null` if no picture is embedded
    ///
    /// @throws If no file or buffer loaded
    ///
    /// @note Returns all embedded pictures including album art, artist photos, etc.
    #[napi(getter)]
    pub fn pictures(&self) -> Result<Option<Vec<MetaPicture>>> {
        self.try_tag(|tag| from_lofty_picture_slice(tag.pictures()))
    }

    #[napi(setter)]
    pub fn set_pictures(&mut self, pictures: Either<Vec<&MetaPicture>, Null>) -> Result<()> {
        self.try_tag_mut(|tag| {
            let new_pics = match pictures {
                Either::A(pics) => pics,
                Either::B(_) => Vec::new(),
            };
            let new_len = new_pics.len();
            let old_len = tag.picture_count() as usize;

            if new_len < old_len {
                for (i, pic) in new_pics.into_iter().enumerate() {
                    tag.set_picture(i, to_lofty_picture(pic));
                }
                (new_len..old_len).rev().for_each(|i| {
                    tag.remove_picture(i);
                });
            } else {
                for (i, pic) in new_pics.into_iter().enumerate() {
                    // lofty handle the index out of bound here
                    tag.set_picture(i, to_lofty_picture(pic));
                }
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
