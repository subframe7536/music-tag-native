use lofty::file::{AudioFile, FileType, TaggedFileExt};
use napi_derive::napi;

use crate::music_file::{MusicFile, MusicFileInner};

#[napi]
impl MusicFile {
    /// Audio quality classification ("HQ", "SQ", or "HiRes")
    ///
    /// Quality is determined based on file format, sample rate, and bit depth:
    /// - HQ: Lossy formats (MP3, AAC, Opus, etc.)
    /// - SQ: Lossless formats up to CD quality (16-bit, up to 48kHz)
    /// - HiRes: Lossless formats with more than 16-bit depth or more than 48kHz sample rate
    ///
    /// MP4 files count as lossless when they carry a bit depth, which lofty only
    /// reports for ALAC and FLAC streams.
    #[napi(getter, ts_return_type = r#""HQ" | "SQ" | "HiRes""#)]
    pub fn quality(&self) -> &str {
        let is_lossless = match self.file.file_type() {
            FileType::Flac | FileType::Ape | FileType::Aiff | FileType::Wav | FileType::WavPack => {
                true
            }
            FileType::Mp4 => self.bit_depth().is_some(),
            _ => false,
        };

        if !is_lossless {
            return "HQ";
        }

        let hi_res_depth = self.bit_depth().is_some_and(|bd| bd > 16);
        let hi_res_rate = self.sample_rate().is_some_and(|sr| sr > 48_000);
        if hi_res_depth || hi_res_rate {
            "HiRes"
        } else {
            "SQ"
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
    /// The file size is the one measured when the file was loaded.
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
            #[cfg(not(target_arch = "wasm32"))]
            MusicFileInner::Path { file_size, .. } => *file_size as f64,
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
    pub fn duration(&self) -> f64 {
        self.file.properties().duration().as_secs_f64() * 1000.0
    }
}
