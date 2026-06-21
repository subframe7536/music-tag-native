use lofty::file::{AudioFile, FileType, TaggedFileExt};
use napi::Result;
use napi_derive::napi;

use crate::tagged_file::{TaggedFile, TaggedFileInner};

#[napi]
impl TaggedFile {
    /// Audio quality classification ("HQ", "SQ", or "HiRes")
    ///
    /// Quality is determined based on file format, sample rate, and bit depth:
    /// - HQ: Lossy formats (MP3, AAC, etc.)
    /// - SQ: Lossless formats at CD quality (44.1kHz, 16-bit)
    /// - HiRes: Lossless formats exceeding CD quality (>44.1kHz, >=16-bit)
    #[napi(getter, ts_return_type = r#""HQ" | "SQ" | "HiRes""#)]
    pub fn quality(&self) -> Result<&str> {
        let is_lossless = matches!(
            self.file.file_type(),
            FileType::Flac | FileType::Ape | FileType::Aiff | FileType::Wav | FileType::WavPack
        );

        if !is_lossless {
            Ok("HQ")
        } else {
            match (self.sample_rate()?, self.bit_depth()?) {
                (Some(sr), Some(bd)) if sr > 44100 && bd >= 16 => Ok("HiRes"),
                _ => Ok("SQ"),
            }
        }
    }

    /// Audio bit depth in bits, or `null` if not available
    ///
    /// Common values: 16 (CD quality), 24 (Hi-Res), 32 (studio quality)
    #[napi(getter)]
    pub fn bit_depth(&self) -> Result<Option<u8>> {
        Ok(self.file.properties().bit_depth())
    }

    /// Audio bit rate in kbps, or `null` if not available
    ///
    /// @note If the audio properties don't provide a bitrate, this method calculates
    /// an approximate bitrate based on file size and duration, including metadata.
    /// The calculated bitrate is constrained between MIN_BITRATE and MAX_BITRATE.
    #[napi(getter)]
    pub fn bit_rate(&self) -> Result<Option<u32>> {
        if let Some(bitrate) = self.file.properties().audio_bitrate() {
            return Ok(Some(bitrate));
        }

        let duration = self.file.properties().duration();
        if duration.is_zero() {
            return Ok(None);
        }

        let duration_secs = duration.as_secs_f64();
        if duration_secs <= f64::EPSILON {
            return Ok(None);
        }

        let file_size_bytes = match &self.inner {
            TaggedFileInner::Buffer => return Ok(None),
            TaggedFileInner::Path(path) => std::fs::metadata(path)
                .map(|m| m.len() as f64)
                .unwrap_or(0.0),
        };
        let bitrate_kbps = ((file_size_bytes * 8.0) / (duration_secs * 1000.0)).round() as u32;

        Ok((8..=10_000).contains(&bitrate_kbps).then_some(bitrate_kbps))
    }

    /// Audio sample rate in Hz, or `null` if not available
    ///
    /// Common values: 44100 (CD), 48000 (DVD), 96000, 192000 (Hi-Res)
    #[napi(getter)]
    pub fn sample_rate(&self) -> Result<Option<u32>> {
        Ok(self.file.properties().sample_rate())
    }

    /// Number of audio channels, or `null` if not available
    ///
    /// Common values: 1 (mono), 2 (stereo), 6 (5.1 surround)
    /// Common values: 1 (mono), 2 (stereo), 6 (5.1 surround), 8 (7.1 surround)
    #[napi(getter)]
    pub fn channels(&self) -> Result<Option<u8>> {
        Ok(self.file.properties().channels())
    }

    /// Audio duration in seconds, or `null` if not available
    #[napi(getter)]
    pub fn duration(&self) -> Result<u32> {
        Ok(self.file.properties().duration().as_millis() as u32)
    }
}
