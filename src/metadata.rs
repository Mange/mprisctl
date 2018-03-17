extern crate dbus;
extern crate mpris;
extern crate serde;
extern crate serde_json;

use std::fmt::Display;
use std::collections::HashMap;
use super::{Error, Settings};
use clap::ArgMatches;
use self::dbus::arg::{ArgType, RefArg};
use self::serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Clone, Copy)]
enum Format {
    Text,
    JSON,
}

#[derive(Debug)]
pub(crate) enum MetadataValue {
    String(String),
    Int64(i64),
    // Not yet supported due to limitations on dbus::arg::ArgRef :(
    // Int16(i16),
    // UInt16(u16),
    // Int32(i32),
    // UInt32(u32),
    // UInt64(u64),
    // Double(f64),
    // Boolean(bool),
    Array(Vec<MetadataValue>),
}

impl MetadataValue {
    fn try_from(arg: &RefArg) -> Option<MetadataValue> {
        match arg.arg_type() {
            ArgType::String => arg.as_str().map(|s| MetadataValue::String(String::from(s))),
            ArgType::Int64 => arg.as_i64().map(MetadataValue::Int64),
            ArgType::Array => arg.as_iter()
                .map(|iter| MetadataValue::Array(iter.flat_map(MetadataValue::try_from).collect())),
            _ => None,
        }
    }
}

impl Serialize for MetadataValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            MetadataValue::String(ref s) => serializer.serialize_str(s),
            MetadataValue::Int64(ref i) => serializer.serialize_i64(*i),
            MetadataValue::Array(ref arr) => {
                use metadata::serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr.iter() {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MetadataView<'a> {
    album_artists: Option<&'a Vec<String>>,
    album_artists_string: Option<String>,
    album_name: Option<&'a str>,
    art_url: Option<&'a str>,
    artists: Option<&'a Vec<String>>,
    artists_string: Option<String>,
    auto_rating: Option<f64>,
    disc_number: Option<i32>,
    length_in_microseconds: Option<u64>,
    length_in_seconds: Option<u64>,
    loop_status: &'static str,
    playback_rate: f64,
    playback_status: &'static str,
    position_in_microseconds: u64,
    position_in_seconds: u64,
    title: Option<&'a str>,
    track_id: &'a str,
    track_number: Option<i32>,
    url: Option<&'a str>,
    volume: f64,

    is_looping_playlist: bool,
    is_looping_track: bool,
    is_paused: bool,
    is_playing: bool,
    is_shuffled: bool,
    is_stopped: bool,

    rest: HashMap<String, MetadataValue>,
}

impl<'a> MetadataView<'a> {
    pub(crate) fn from_player(
        metadata: &'a mpris::Metadata,
        player: &'a mpris::Player,
    ) -> Result<MetadataView<'a>, mpris::DBusError> {
        use mpris::PlaybackStatus::*;
        use mpris::LoopStatus;

        let playback_status = player.get_playback_status()?;
        let playback_status_str = match playback_status {
            Playing => "Playing",
            Paused => "Paused",
            Stopped => "Stopped",
        };

        let loop_status = player.get_loop_status()?;
        let loop_status_str = match loop_status {
            LoopStatus::None => "None",
            LoopStatus::Track => "Track",
            LoopStatus::Playlist => "Playlist",
        };

        let position = player.get_position()?;
        let position_in_microseconds =
            position.as_secs() + position.subsec_nanos() as u64 * 1_000_000_000;
        let position_in_seconds = position.as_secs();

        let playback_rate = player.get_playback_rate()?;
        let shuffled = player.get_shuffle()?;
        let volume = player.get_volume()?;

        Ok(MetadataView {
            album_artists: metadata.album_artists(),
            album_artists_string: metadata.album_artists().map(|a| a.join(", ")),
            album_name: metadata.album_name(),
            art_url: metadata.art_url(),
            artists: metadata.artists(),
            artists_string: metadata.artists().map(|a| a.join(", ")),
            auto_rating: metadata.auto_rating(),
            disc_number: metadata.disc_number(),
            length_in_microseconds: metadata.length_in_microseconds(),
            length_in_seconds: metadata.length_in_microseconds().map(|us| us / 1000 / 1000),
            loop_status: loop_status_str,
            playback_rate,
            playback_status: playback_status_str,
            position_in_microseconds,
            position_in_seconds,
            title: metadata.title(),
            track_id: metadata.track_id(),
            track_number: metadata.track_number(),
            url: metadata.url(),
            volume,

            is_looping_playlist: loop_status == LoopStatus::Playlist,
            is_looping_track: loop_status == LoopStatus::Track,
            is_playing: playback_status == Playing,
            is_shuffled: shuffled,
            is_paused: playback_status == Paused,
            is_stopped: playback_status == Stopped,

            rest: metadata
                .rest()
                .iter()
                .flat_map(|(key, value)| {
                    if let Some(val) = MetadataValue::try_from(value) {
                        Some((key.to_owned(), val))
                    } else {
                        None
                    }
                })
                .collect(),
        })
    }
}

pub(crate) fn run(matches: Option<&ArgMatches>, settings: &Settings) -> Result<(), Error> {
    let player = settings.find_player()?;
    let metadata = player.get_metadata()?;
    let metadata_view = MetadataView::from_player(&metadata, &player)?;

    let format = match matches {
        Some(matches) if matches.is_present("json") => Format::JSON,
        _ => Format::Text,
    };

    match format {
        Format::Text => print_metadata(&metadata_view),
        Format::JSON => match serde_json::to_string(&metadata_view) {
            Ok(json) => {
                println!("{}", json);
                Ok(())
            }
            Err(error) => Err(Error::from(error)),
        },
    }
}

fn print_metadata<'a>(view: &'a MetadataView<'a>) -> Result<(), Error> {
    print_text_field("Playback status", &Some(view.playback_status));
    print_text_field("Track ID", &Some(view.track_id));
    print_text_field("Title", &view.title);
    print_text_field("Artists", &view.artists_string);
    print_text_field("Album", &view.album_name);
    print_text_field("Track number", &view.track_number);
    print_text_field("Album artists", &view.album_artists_string);
    print_text_field("Artwork URL", &view.art_url);
    print_text_field("Auto-rating", &view.auto_rating);
    print_text_field("Disc number", &view.disc_number);
    print_text_field("URL", &view.url);
    print_text_field("Length (µs)", &view.length_in_microseconds);
    print_text_field("Length (s)", &view.length_in_seconds);
    print_text_field("Playback rate", &Some(view.playback_rate));
    print_text_field("Position (µs)", &Some(view.position_in_microseconds));
    print_text_field("Position (s)", &Some(view.position_in_seconds));
    print_text_field("Looping", &Some(view.loop_status));
    print_text_field("Shuffled", &Some(view.is_shuffled));
    print_text_field("Volume (unitless)", &Some(view.volume));
    Ok(())
}

// Length of longest text field text ("Volume (unitless)")
const TEXT_FIELD_PADDING: usize = 17;

fn print_text_field<T: Display>(title: &'static str, value: &Option<T>) {
    match *value {
        Some(ref val) => println!(
            "{title:width$}\t{value}",
            title = title,
            value = val,
            width = TEXT_FIELD_PADDING
        ),
        None => println!("{}:", title),
    }
}
