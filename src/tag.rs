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
    helper::{insert_text_checked, tag_type_name},
    meta_picture::{from_lofty_picture_slice, to_lofty_picture, MetaPicture},
    music_file::MusicFile,
    utils::{format_replaygain_gain, format_replaygain_peak, parse_replaygain_value},
};

const ERR_INVALID_RATING: &str = "Rating should be integer in [1, 5]";

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Replace the year of an existing timestamp, keeping the other components.
/// A day that no longer exists in the new year (Feb 29) is clamped.
fn with_year(existing: Option<Timestamp>, year: u16) -> Timestamp {
    let mut date = existing.unwrap_or_default();
    date.year = year;
    if let (Some(month), Some(day)) = (date.month, date.day) {
        date.day = Some(day.min(days_in_month(year, month)));
    }
    date
}

#[napi]
impl MusicFile {
    /// File's metadata tag type, or `null` if not recognized or no available tag
    #[napi(
        getter,
        ts_return_type = r#""AIFF" | "APE" | "ID3V1" | "ID3V2" | "ILST" | "RIFF" | "VORBIS" | null"#
    )]
    pub fn tag_type(&self) -> Option<String> {
        self.tag(|tag| tag_type_name(tag.tag_type()).map(String::from))
    }

    /// Title, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn title(&self) -> Option<String> {
        self.tag(|tag| tag.title().map(String::from))
    }

    #[napi(setter)]
    pub fn set_title(&mut self, title: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::TrackTitle, "Title", title)
    }

    /// Artist, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn artist(&self) -> Option<String> {
        self.tag(|tag| tag.artist().map(String::from))
    }

    #[napi(setter)]
    pub fn set_artist(&mut self, artist: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::TrackArtist, "Artist", artist)
    }

    /// Album, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn album(&self) -> Option<String> {
        self.tag(|tag| tag.album().map(String::from))
    }

    #[napi(setter)]
    pub fn set_album(&mut self, album: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::AlbumTitle, "Album", album)
    }

    /// Year, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn year(&self) -> Option<u16> {
        self.tag(|tag| tag.date().map(|d| d.year))
    }

    #[napi(setter)]
    pub fn set_year(&mut self, year: Either<f64, Null>) -> Result<()> {
        let year = match year {
            Either::A(year) => {
                Some(Self::validate_integer(year, "Year", 0, u16::MAX as u64)? as u16)
            }
            Either::B(_) => None,
        };

        self.tag_mut(|tag| match year {
            Some(year) => {
                // Keep the existing month/day/time, only replace the year.
                let date = with_year(tag.date(), year);
                if tag.insert_text(ItemKey::RecordingDate, date.to_string()) {
                    tag.remove_key(ItemKey::Year);
                    Ok(())
                } else {
                    insert_text_checked(tag, ItemKey::Year, "Year", year.to_string())
                }
            }
            None => {
                tag.remove_date();
                Ok(())
            }
        })
    }

    /// Genre, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn genre(&self) -> Option<String> {
        self.tag(|tag| tag.genre().map(String::from))
    }

    #[napi(setter)]
    pub fn set_genre(&mut self, genre: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Genre, "Genre", genre)
    }

    /// Track number, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn track_number(&self) -> Option<u32> {
        self.tag(|tag| tag.track())
    }

    #[napi(setter)]
    pub fn set_track_number(&mut self, track_number: Either<f64, Null>) -> Result<()> {
        self.set_integer_field(ItemKey::TrackNumber, "Track number", track_number)
    }

    /// Disc number, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn disc_number(&self) -> Option<u32> {
        self.tag(|tag| tag.disk())
    }

    #[napi(setter)]
    pub fn set_disc_number(&mut self, disc_number: Either<f64, Null>) -> Result<()> {
        self.set_integer_field(ItemKey::DiscNumber, "Disc number", disc_number)
    }

    /// Total number of tracks in the album, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn track_total(&self) -> Option<u32> {
        self.tag(|tag| tag.track_total())
    }

    #[napi(setter)]
    pub fn set_track_total(&mut self, track_total: Either<f64, Null>) -> Result<()> {
        self.set_integer_field(ItemKey::TrackTotal, "Track total", track_total)
    }

    /// Total number of discs in the album, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn discs_total(&self) -> Option<u32> {
        self.tag(|tag| tag.disk_total())
    }

    #[napi(setter)]
    pub fn set_discs_total(&mut self, discs_total: Either<f64, Null>) -> Result<()> {
        self.set_integer_field(ItemKey::DiscTotal, "Disc total", discs_total)
    }

    /// Comment, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn comment(&self) -> Option<String> {
        self.tag(|tag| tag.comment().map(String::from))
    }

    #[napi(setter)]
    pub fn set_comment(&mut self, comment: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Comment, "Comment", comment)
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
        self.set_text_field(ItemKey::AlbumArtist, "Album artist", album_artist)
    }

    /// Composer, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn composer(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Composer).map(String::from))
    }

    #[napi(setter)]
    pub fn set_composer(&mut self, composer: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Composer, "Composer", composer)
    }

    /// Conductor, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn conductor(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Conductor).map(String::from))
    }

    #[napi(setter)]
    pub fn set_conductor(&mut self, conductor: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Conductor, "Conductor", conductor)
    }

    /// Lyricist, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn lyricist(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Lyricist).map(String::from))
    }

    #[napi(setter)]
    pub fn set_lyricist(&mut self, lyricist: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Lyricist, "Lyricist", lyricist)
    }

    /// Publisher, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn publisher(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::Publisher).map(String::from))
    }

    #[napi(setter)]
    pub fn set_publisher(&mut self, publisher: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::Publisher, "Publisher", publisher)
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
        let lyrics = match lyrics {
            Either::A(lyrics) => Some(lyrics),
            Either::B(_) => None,
        };

        // Pick the key from the tag that is actually written, which may be a
        // newly created primary tag rather than the one currently read from.
        self.tag_mut(|tag| {
            let key = match tag.tag_type() {
                LoftyTagType::Id3v2 => ItemKey::UnsyncLyrics,
                _ => ItemKey::Lyrics,
            };
            match &lyrics {
                Some(lyrics) => insert_text_checked(tag, key, "Lyrics", lyrics.clone()),
                None => {
                    tag.remove_key(key);
                    Ok(())
                }
            }
        })
    }

    /// Copyright information, or `null` if not set or no available tag
    #[napi(getter)]
    pub fn copyright(&self) -> Option<String> {
        self.tag(|tag| tag.get_string(ItemKey::CopyrightMessage).map(String::from))
    }

    #[napi(setter)]
    pub fn set_copyright(&mut self, copyright: Either<String, Null>) -> Result<()> {
        self.set_text_field(ItemKey::CopyrightMessage, "Copyright", copyright)
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
    pub fn set_rating(&mut self, rating: Either<f64, Null>) -> Result<()> {
        let star_rating = match rating {
            Either::A(value) => {
                let value = Self::validate_integer(value, "Rating", 1, 5)
                    .map_err(|_| Error::new(Status::InvalidArg, ERR_INVALID_RATING))?;
                Some(match value {
                    1 => StarRating::One,
                    2 => StarRating::Two,
                    3 => StarRating::Three,
                    4 => StarRating::Four,
                    5 => StarRating::Five,
                    _ => unreachable!("rating validation guarantees the range"),
                })
            }
            Either::B(_) => None,
        };

        self.tag_mut(move |tag| match star_rating {
            Some(value) => insert_text_checked(
                tag,
                ItemKey::Popularimeter,
                "Rating",
                Popularimeter::custom("", value, 0).to_string(),
            ),
            None => {
                tag.remove_key(ItemKey::Popularimeter);
                Ok(())
            }
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
            "Track replay gain",
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
            "Track replay peak",
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
            "Album replay gain",
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
            "Album replay peak",
            album_replay_peak,
            format_replaygain_peak,
        )
    }

    /// Embedded pictures/album art list, or `null` if no picture is embedded or no available tag
    ///
    /// @note Returns all embedded pictures including album art, artist photos, etc.
    /// @note Each access copies every picture's data. Store the result in a variable
    /// instead of reading this property repeatedly.
    #[napi(getter)]
    pub fn pictures(&self) -> Option<Vec<MetaPicture>> {
        self.tag(|tag| from_lofty_picture_slice(tag.pictures()))
    }

    #[napi(setter)]
    pub fn set_pictures(&mut self, pictures: Either<Vec<&MetaPicture>, Null>) -> Result<()> {
        let new_pics = match pictures {
            Either::A(pics) => pics,
            Either::B(_) => Vec::new(),
        };

        self.tag_mut(|tag| {
            let new_len = new_pics.len();
            let old_len = tag.picture_count() as usize;

            for (i, pic) in new_pics.iter().enumerate() {
                // lofty handles the index out of bounds here (appends when i >= picture_count)
                tag.set_picture(i, to_lofty_picture(pic));
            }

            if new_len < old_len {
                (new_len..old_len).rev().for_each(|i| {
                    tag.remove_picture(i);
                });
            }

            Ok(())
        })
    }
}
