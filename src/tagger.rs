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
const ERR_NO_BUFFER: &str = "Tagger's buffer is empty";
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

    /// Load music file from a byte buffer
    ///
    /// @param buffer A Uint8Array containing the audio file data
    ///
    /// @throws If the buffer doesn't contain a valid audio file
    /// @throws If the file doesn't contain any metadata tags
    ///
    /// @note This method disposes any previously loaded file before loading the new one.
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

    /// Load music file from a file path
    ///
    /// @param path The file system path to the audio file
    ///
    /// @throws If the path doesn't exist or isn't accessible
    /// @throws If the file doesn't contain a valid audio format
    /// @throws If the file doesn't contain any metadata tags
    ///
    /// @note This method disposes any previously loaded file before loading the new one
    /// @note Not available in WebAssembly environments due to file system restrictions.
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
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    /// @throws If the file was loaded from a path (`ERR_NO_BUFFER`)
    /// @throws If saving fails due to file format constraints
    ///
    /// @note This method only works for files loaded via `loadBuffer()`
    /// @note After saving, the modified buffer can be retrieved using the `buffer` getter
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

    /// Save metadata changes to a file path
    ///
    /// @param path Optional file path to save the file. If `undefined`, saves to the original path
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    /// @throws If no path is provided and file wasn't loaded from a path (`ERR_NO_BUFFER`)
    /// @throws When used in WebAssembly environments
    /// @throws If saving fails due to file system or format constraints
    ///
    /// @note Not available in WebAssembly environments due to file system restrictions
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

    /// Get the audio file buffer with current metadata
    ///
    /// @returns The audio file data as a byte array
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// @note For files loaded via `loadBuffer()`, call `saveBuffer()` first to ensure
    /// metadata changes are applied. For files loaded via `loadPath()`, this
    /// returns an empty buffer.
    #[napi(getter)]
    pub fn buffer(&self) -> Result<Uint8Array> {
        Ok(Uint8Array::new(self.inner()?.buffer.clone()))
    }

    /// Get the audio file quality classification
    ///
    /// @returns The quality classification ("HQ", "SQ", or "HiRes")
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the audio bit depth
    ///
    /// @returns Bit depth in bits, or `null` if not available
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// Common values: 16 (CD quality), 24 (Hi-Res), 32 (studio quality)
    #[napi(getter)]
    pub fn bit_depth(&self) -> Result<Option<u8>> {
        Ok(self.inner()?.file.properties().bit_depth())
    }

    /// Get the audio bit rate
    ///
    /// @returns Bit rate in kbps, or `null` if not available
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the audio sample rate
    ///
    /// @returns Sample rate in Hz, or `null` if not available
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// Common values: 44100 (CD), 48000 (DVD), 96000, 192000 (Hi-Res)
    #[napi(getter)]
    pub fn sample_rate(&self) -> Result<Option<u32>> {
        Ok(self.inner()?.file.properties().sample_rate())
    }

    /// Get the number of audio channels
    ///
    /// @returns Number of channels, or `null` if not available
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// Common values: 1 (mono), 2 (stereo), 6 (5.1 surround)
    /// Common values: 1 (mono), 2 (stereo), 6 (5.1 surround), 8 (7.1 surround)
    #[napi(getter)]
    pub fn channels(&self) -> Result<Option<u8>> {
        Ok(self.inner()?.file.properties().channels())
    }

    /// Get the audio duration
    ///
    /// @returns Duration in seconds, or `null` if not available
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn duration(&self) -> Result<u32> {
        Ok(self.inner()?.file.properties().duration().as_millis() as u32)
    }

    /// Get the file's metadata tag type
    ///
    /// Supported tag types: "ID3V1", "ID3V2", "APE", "VORBIS", "MP4", "AIFF", "RIFF"
    ///
    /// @returns Tag type as string, or `null` if not recognized
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the title metadata
    ///
    /// @returns Track title, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn title(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.title().map(String::from))
    }

    #[napi(setter)]
    pub fn set_title(&mut self, title: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_title(title))
    }

    /// Get the artist metadata
    ///
    /// @returns Track artist, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn artist(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.artist().map(String::from))
    }

    #[napi(setter)]
    pub fn set_artist(&mut self, artist: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_artist(artist))
    }

    /// Get the album metadata
    ///
    /// @returns Album name, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn album(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.album().map(String::from))
    }

    #[napi(setter)]
    pub fn set_album(&mut self, album: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_album(album))
    }

    /// Get the year metadata
    ///
    /// @returns Release year, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn year(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.year())
    }

    #[napi(setter)]
    pub fn set_year(&mut self, year: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_year(year))
    }

    /// Get the genre metadata
    ///
    /// @returns Music genre, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn genre(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.genre().map(String::from))
    }

    #[napi(setter)]
    pub fn set_genre(&mut self, genre: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_genre(genre))
    }

    /// Get the track number metadata
    ///
    /// @returns Track number, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn track_number(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.track())
    }

    #[napi(setter)]
    pub fn set_track_number(&mut self, track_number: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_track(track_number))
    }

    /// Get the disc number metadata
    ///
    /// @returns Disc number, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn disc_number(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.disk())
    }

    #[napi(setter)]
    pub fn set_disc_number(&mut self, disc_number: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_disk(disc_number))
    }

    /// Get the total number of tracks in the album
    ///
    /// @returns Total track count, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn track_total(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.track_total())
    }

    #[napi(setter)]
    pub fn set_track_total(&mut self, track_total: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_track_total(track_total))
    }

    /// Get the total number of discs in the album
    ///
    /// @returns Total disc count, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn discs_total(&self) -> Result<Option<u32>> {
        self.try_tag(|tag| tag.disk_total())
    }

    #[napi(setter)]
    pub fn set_discs_total(&mut self, discs_total: u32) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_disk_total(discs_total))
    }

    /// Get the comment metadata
    ///
    /// @returns User comment, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    #[napi(getter)]
    pub fn comment(&self) -> Result<Option<String>> {
        self.try_tag(|tag| tag.comment().map(String::from))
    }

    #[napi(setter)]
    pub fn set_comment(&mut self, comment: String) -> Result<()> {
        self.try_tag_mut(|tag| tag.set_comment(comment))
    }

    /// Get the album artist metadata
    ///
    /// @returns Album artist, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// @note Album artist differs from track artist and represents the primary artist for the entire album.
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

    /// Get the composer metadata
    ///
    /// @returns Music composer, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the conductor metadata
    ///
    /// @returns Orchestra conductor, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the lyricist metadata
    ///
    /// @returns Song lyricist, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the publisher metadata
    ///
    /// @returns Music publisher, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the lyrics metadata
    ///
    /// @returns Song lyrics, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the copyright metadata
    ///
    /// @returns Copyright information, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
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

    /// Get the track replay gain metadata
    ///
    /// @returns Track replay gain in dB, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// @note Replay gain is used to normalize playback volume across different tracks.
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

    /// Get the track replay peak metadata
    ///
    /// @returns Track replay peak value, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// @note Replay peak represents the maximum amplitude level in the track.
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

    /// Get the album replay gain metadata
    ///
    /// @returns Album replay gain in dB, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// @note Album replay gain normalizes playback volume across different albums.
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

    /// Get the album replay peak metadata
    ///
    /// @returns Album replay peak value, or `null` if not set
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// @note Album replay peak represents the maximum amplitude level in the album.
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

    /// Get the embedded pictures/album art from the music file
    ///
    /// @returns Array of embedded pictures, or `null` if no pictures are embedded
    ///
    /// @throws If no file is loaded (`ERR_DISPOSED`)
    ///
    /// @note Returns all embedded pictures including album art, artist photos, etc.
    /// @note Each picture includes metadata like picture type and image data.
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
