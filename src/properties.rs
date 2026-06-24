use lofty::file::{AudioFile, FileType, TaggedFileExt};
use napi_derive::napi;

use crate::music_file::{MusicFile, MusicFileInner};

#[napi]
impl MusicFile {
    /// Audio quality classification ("HQ", "SQ", or "HiRes")
    ///
    /// Quality is determined based on file format, sample rate, and bit depth:
    /// - HQ: Lossy formats (MP3, AAC, etc.)
    /// - SQ: Lossless formats at CD quality (44.1kHz, 16-bit)
    /// - HiRes: Lossless formats exceeding CD quality (>44.1kHz, >=16-bit)
    #[napi(getter, ts_return_type = r#""HQ" | "SQ" | "HiRes""#)]
    pub fn quality(&self) -> &str {
        let is_lossless = matches!(
            self.file.file_type(),
            FileType::Flac | FileType::Ape | FileType::Aiff | FileType::Wav | FileType::WavPack
        );

        if !is_lossless {
            "HQ"
        } else {
            match (self.sample_rate(), self.bit_depth()) {
                (Some(sr), Some(bd)) if sr > 44100 && bd >= 16 => "HiRes",
                _ => "SQ",
            }
        }
    }

    /// Audio bit depth in bits, or `null` if not available
    ///
    /// Common values: 16 (CD quality), 24 (Hi-Res), 32 (studio quality)
    #[napi(getter)]
    pub fn bit_depth(&self) -> Option<u8> {
        self.file.properties().bit_depth()
    }

    /// Audio bit rate in kbps, or `null` if not available
    ///
    /// @note If the audio properties don't provide a bitrate, this method calculates
    /// an approximate bitrate based on file size and duration, including metadata.
    /// The calculated bitrate is constrained between 8 and 10,000 kbps.
    #[napi(getter)]
    pub fn bit_rate(&self) -> Option<u32> {
        if let Some(bitrate) = self.file.properties().audio_bitrate() {
            return Some(bitrate);
        }

        let duration = self.file.properties().duration();
        if duration.is_zero() {
            return None;
        }

        let duration_secs = duration.as_secs_f64();
        if duration_secs <= f64::EPSILON {
            return None;
        }

        let file_size_bytes = match &self.inner {
            MusicFileInner::Buffer { source_len } => *source_len as f64,
            MusicFileInner::Path(path) => std::fs::metadata(path)
                .map(|m| m.len() as f64)
                .unwrap_or(0.0),
        };
        let bitrate_kbps = ((file_size_bytes * 8.0) / (duration_secs * 1000.0)).round() as u32;

        (8..=10_000).contains(&bitrate_kbps).then_some(bitrate_kbps)
    }

    /// Audio sample rate in Hz, or `null` if not available
    ///
    /// Common values: 44100 (CD), 48000 (DVD), 96000, 192000 (Hi-Res)
    #[napi(getter)]
    pub fn sample_rate(&self) -> Option<u32> {
        self.file.properties().sample_rate()
    }

    /// Number of audio channels, or `null` if not available
    ///
    /// Common values: 1 (mono), 2 (stereo), 6 (5.1 surround), 8 (7.1 surround)
    #[napi(getter)]
    pub fn channels(&self) -> Option<u8> {
        self.file.properties().channels()
    }

    /// Audio duration in milliseconds, 0 if not available
    #[napi(getter)]
    pub fn duration(&self) -> u32 {
        self.file.properties().duration().as_millis() as u32
    }
}
