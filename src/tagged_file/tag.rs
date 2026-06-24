use lofty::tag::{
    items::{
        popularimeter::{Popularimeter, StarRating},
        Timestamp,
    },
    Accessor, ItemKey, TagType as LoftyTagType,
};
use napi::{bindgen_prelude::Null, Either, Error, Result, Status};
use napi_derive::napi;

use crate::{
    meta_picture::{from_lofty_picture_slice, to_lofty_picture, MetaPicture},
    tagged_file::TaggedFile,
    utils::{format_replaygain_gain, format_replaygain_peak, parse_replaygain_value},
};

const ERR_INVALID_RATING: &str = "Rating should be integer in [1, 5]";

#[napi]
impl TaggedFile {
    /// File's metadata tag type, or `null` if not recognized or no available tag
    #[napi(
        getter,
        ts_return_type = r#""AIFF" | "APE" | "ID3V1" | "ID3V2" | "ILST" | "RIFF" | "VORBIS" | null"#
    )]
    pub fn tag_type(&self) -> Option<String> {
        self.tag(|tag| match tag.tag_type() {
            LoftyTagType::AiffText => Some("AIFF".to_string()),
            LoftyTagType::Ape => Some("APE".to_string()),
            LoftyTagType::Id3v1 => Some("ID3V1".to_string()),
            LoftyTagType::Id3v2 => Some("ID3V2".to_string()),
            LoftyTagType::Mp4Ilst => Some("ILST".to_string()),
            LoftyTagType::RiffInfo => Some("RIFF".to_string()),
            LoftyTagType::VorbisComments => Some("VORBIS".to_string()),
            _ => None,
        })
    }

    /// Title, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn title(&self) -> Option<String> {
        self.tag(|tag| tag.title().map(String::from))
    }

    #[napi(setter)]
    pub fn set_title(&mut self, title: Either<String, Null>) -> Result<()> {
        self.tag_mut(|tag| match title {
            Either::A(t) => tag.set_title(t),
            _ => tag.remove_title(),
        })
    }

    /// Artist, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn artist(&self) -> Option<String> {
        self.tag(|tag| tag.artist().map(String::from))
    }

    #[napi(setter)]
    pub fn set_artist(&mut self, artist: Either<String, Null>) -> Result<()> {
        self.tag_mut(|tag| match artist {
            Either::A(a) => tag.set_artist(a),
            _ => tag.remove_artist(),
        })
    }

    /// Album, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn album(&self) -> Option<String> {
        self.tag(|tag| tag.album().map(String::from))
    }

    #[napi(setter)]
    pub fn set_album(&mut self, album: Either<String, Null>) -> Result<()> {
        self.tag_mut(|tag| match album {
            Either::A(a) => tag.set_album(a),
            _ => tag.remove_album(),
        })
    }

    /// Year, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn year(&self) -> Option<u16> {
        self.tag(|tag| tag.date().map(|d| d.year))
    }

    #[napi(setter)]
    pub fn set_year(&mut self, year: Either<u16, Null>) -> Result<()> {
        self.tag_mut(|tag| match year {
            Either::A(y) => tag.set_date(Timestamp {
                year: y,
                ..Default::default()
            }),
            _ => tag.remove_date(),
        })
    }

    /// Genre, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn genre(&self) -> Option<String> {
        self.tag(|tag| tag.genre().map(String::from))
    }

    #[napi(setter)]
    pub fn set_genre(&mut self, genre: Either<String, Null>) -> Result<()> {
        self.tag_mut(|tag| match genre {
            Either::A(g) => tag.set_genre(g),
            _ => tag.remove_genre(),
        })
    }

    /// Track number, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn track_number(&self) -> Option<u32> {
        self.tag(|tag| tag.track())
    }

    #[napi(setter)]
    pub fn set_track_number(&mut self, track_number: Either<u32, Null>) -> Result<()> {
        self.tag_mut(|tag| match track_number {
            Either::A(t) => tag.set_track(t),
            _ => tag.remove_track(),
        })
    }

    /// Disc number, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn disc_number(&self) -> Option<u32> {
        self.tag(|tag| tag.disk())
    }

    #[napi(setter)]
    pub fn set_disc_number(&mut self, disc_number: Either<u32, Null>) -> Result<()> {
        self.tag_mut(|tag| match disc_number {
            Either::A(d) => tag.set_disk(d),
            _ => tag.remove_disk(),
        })
    }

    /// Total number of tracks in the album, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn track_total(&self) -> Option<u32> {
        self.tag(|tag| tag.track_total())
    }

    #[napi(setter)]
    pub fn set_track_total(&mut self, track_total: Either<u32, Null>) -> Result<()> {
        self.tag_mut(|tag| match track_total {
            Either::A(t) => tag.set_track_total(t),
            _ => tag.remove_track_total(),
        })
    }

    /// Total number of discs in the album, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn discs_total(&self) -> Option<u32> {
        self.tag(|tag| tag.disk_total())
    }

    #[napi(setter)]
    pub fn set_discs_total(&mut self, discs_total: Either<u32, Null>) -> Result<()> {
        self.tag_mut(|tag| match discs_total {
            Either::A(d) => tag.set_disk_total(d),
            _ => tag.remove_disk_total(),
        })
    }

    /// Comment, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn comment(&self) -> Option<String> {
        self.tag(|tag| tag.comment().map(String::from))
    }

    #[napi(setter)]
    pub fn set_comment(&mut self, comment: Either<String, Null>) -> Result<()> {
        self.tag_mut(|tag| match comment {
            Either::A(c) => tag.set_comment(c),
            _ => tag.remove_comment(),
        })
    }

    /// Album artist, or `null` if not set or no available tag
    ///
    /// @note Album artist differs from track artist and represents the primary artist for the entire album.
    #[napi(getter)]
    pub fn album_artist(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::AlbumArtist).map(String::from))
    }

    #[napi(setter)]
    pub fn set_album_artist(&mut self, album_artist: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::AlbumArtist, album_artist)
    }

    /// Composer, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn composer(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Composer).map(String::from))
    }

    #[napi(setter)]
    pub fn set_composer(&mut self, composer: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Composer, composer)
    }

    /// Conductor, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn conductor(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Conductor).map(String::from))
    }

    #[napi(setter)]
    pub fn set_conductor(&mut self, conductor: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Conductor, conductor)
    }

    /// Lyricist, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn lyricist(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Lyricist).map(String::from))
    }

    #[napi(setter)]
    pub fn set_lyricist(&mut self, lyricist: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Lyricist, lyricist)
    }

    /// Publisher, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn publisher(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Publisher).map(String::from))
    }

    #[napi(setter)]
    pub fn set_publisher(&mut self, publisher: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Publisher, publisher)
    }

    /// Lyrics, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn lyrics(&self) -> Option<String> {
        self.tag(|tag| match tag.tag_type() {
            LoftyTagType::Id3v2 => tag.get_string(ItemKey::UnsyncLyrics).map(String::from),
            _ => tag.get_string(ItemKey::Lyrics).map(String::from),
        })
    }

    #[napi(setter)]
    pub fn set_lyrics(&mut self, lyrics: Either<String, Null>) -> Result<()> {
        match self.tag(|tag| Some(tag.tag_type())) {
            Some(LoftyTagType::Id3v2) => self.set_text_field(ItemKey::UnsyncLyrics, lyrics),
            _ => self.set_text_field(ItemKey::Lyrics, lyrics),
        }
    }

    /// Copyright information, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn copyright(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::CopyrightMessage).map(String::from))
    }

    #[napi(setter)]
    pub fn set_copyright(&mut self, copyright: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::CopyrightMessage, copyright)
    }

    /// User star ratings, or `null` if not set or no available tag
    #[napi(getter, ts_return_type = "1 | 2 | 3 | 4 | 5 | null")]
    pub fn rating(&self) -> Option<u8> {
        self.tag(|tag| match tag.ratings().next().map(|p| p.rating) {
            Some(StarRating::One) => Some(1),
            Some(StarRating::Two) => Some(2),
            Some(StarRating::Three) => Some(3),
            Some(StarRating::Four) => Some(4),
            Some(StarRating::Five) => Some(5),
            _ => None,
        })
    }

    #[napi(setter)]
    pub fn set_rating(&mut self, rating: Either<u32, Null>) -> Result<()> {
        let star_rating = match rating {
            Either::A(value) => Some(
                match value {
                    1 => Some(StarRating::One),
                    2 => Some(StarRating::Two),
                    3 => Some(StarRating::Three),
                    4 => Some(StarRating::Four),
                    5 => Some(StarRating::Five),
                    _ => None,
                }
                .ok_or_else(|| Error::new(Status::InvalidArg, ERR_INVALID_RATING))?,
            ),
            Either::B(_) => None,
        };

        self.tag_mut(move |tag| match star_rating {
            Some(value) => {
                tag.insert_text(
                    ItemKey::Popularimeter,
                    Popularimeter::custom("", value, 0).to_string(),
                );
            }
            None => tag.remove_key(ItemKey::Popularimeter),
        })
    }

    /// Track replay gain in dB, or `null` if not set or no available tag
    ///
    /// @note Replay gain is used to normalize playback volume across different tracks.
    #[napi(getter)]
    pub fn track_replay_gain(&self) -> Option<f64> {
        self.tag(|tag| parse_replaygain_value(tag.get_string(ItemKey::ReplayGainTrackGain)?))
    }

    #[napi(setter)]
    pub fn set_track_replay_gain(&mut self, track_replay_gain: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainTrackGain,
            track_replay_gain,
            format_replaygain_gain,
        )
    }

    /// Track replay peak value, or `null` if not set or no available tag
    ///
    /// @note Replay peak represents the maximum amplitude level in the track.
    #[napi(getter)]
    pub fn track_replay_peak(&self) -> Option<f64> {
        self.tag(|tag| parse_replaygain_value(tag.get_string(ItemKey::ReplayGainTrackPeak)?))
    }

    #[napi(setter)]
    pub fn set_track_replay_peak(&mut self, track_replay_peak: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainTrackPeak,
            track_replay_peak,
            format_replaygain_peak,
        )
    }

    /// Album replay gain in dB, or `null` if not set or no available tag
    ///
    /// @note Album replay gain normalizes playback volume across different albums.
    #[napi(getter)]
    pub fn album_replay_gain(&self) -> Option<f64> {
        self.tag(|tag| parse_replaygain_value(tag.get_string(ItemKey::ReplayGainAlbumGain)?))
    }

    #[napi(setter)]
    pub fn set_album_replay_gain(&mut self, album_replay_gain: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainAlbumGain,
            album_replay_gain,
            format_replaygain_gain,
        )
    }

    /// Album replay peak value, or `null` if not set or no available tag
    ///
    /// @note Album replay peak represents the maximum amplitude level in the album.
    #[napi(getter)]
    pub fn album_replay_peak(&self) -> Option<f64> {
        self.tag(|tag| parse_replaygain_value(tag.get_string(ItemKey::ReplayGainAlbumPeak)?))
    }

    #[napi(setter)]
    pub fn set_album_replay_peak(&mut self, album_replay_peak: Either<f64, Null>) -> Result<()> {
        self.set_gain_value(
            ItemKey::ReplayGainAlbumPeak,
            album_replay_peak,
            format_replaygain_peak,
        )
    }

    /// Embedded pictures/album art list, or `null` if no picture is embedded or no available tag
    ///
    /// @note Returns all embedded pictures including album art, artist photos, etc.
    #[napi(getter)]
    pub fn pictures(&self) -> Option<Vec<MetaPicture>> {
        self.tag(|tag| from_lofty_picture_slice(tag.pictures()))
    }

    #[napi(setter)]
    pub fn set_pictures(&mut self, pictures: Either<Vec<&MetaPicture>, Null>) -> Result<()> {
        self.tag_mut(|tag| {
            let new_pics = match pictures {
                Either::A(pics) => pics,
                Either::B(_) => Vec::new(),
            };
            let new_len = new_pics.len();
            let old_len = tag.picture_count() as usize;

            for (i, pic) in new_pics.into_iter().enumerate() {
                // lofty handles the index out of bounds here (appends when i >= picture_count)
                tag.set_picture(i, to_lofty_picture(pic));
            }

            if new_len < old_len {
                (new_len..old_len).rev().for_each(|i| {
                    tag.remove_picture(i);
                });
            }
        })
    }
}
