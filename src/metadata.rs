extern crate mpris;
extern crate serde;
extern crate serde_json;

use super::{Error, Settings};
use std::collections::HashMap;
use std::fmt::Display;
use structopt::StructOpt;

use mpris::{DBusError, LoopStatus, Metadata, PlaybackStatus, Player, Progress, TrackID};

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(
        short = "f",
        long = "format",
        default_value = "text",
        raw(possible_values = "&Format::variants()")
    )]
    /// Render metadata in this format.
    format: Format,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Format {
    Text,
    JSON,
}

impl Format {
    fn variants() -> [&'static str; 2] {
        ["text", "json"]
    }
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match caseless::default_case_fold_str(s).as_str() {
            "text" => Ok(Format::Text),
            "json" => Ok(Format::JSON),
            _ => Err(format!("\"{}\" is not a valid format", s)),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MetadataView<'a> {
    album_artists: Option<Vec<&'a str>>,
    album_artists_string: Option<String>,
    album_name: Option<&'a str>,
    art_url: Option<&'a str>,
    artists: Option<Vec<&'a str>>,
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
    track_id: Option<String>,
    track_number: Option<i32>,
    url: Option<&'a str>,
    volume: f64,

    is_looping_playlist: bool,
    is_looping_track: bool,
    is_paused: bool,
    is_playing: bool,
    is_shuffled: bool,
    is_stopped: bool,

    raw: HashMap<String, serde_json::Value>,
}

fn playback_status_str(playback_status: PlaybackStatus) -> &'static str {
    use self::PlaybackStatus::*;

    match playback_status {
        Playing => "Playing",
        Paused => "Paused",
        Stopped => "Stopped",
    }
}

fn loop_status_str(loop_status: LoopStatus) -> &'static str {
    match loop_status {
        LoopStatus::None => "None",
        LoopStatus::Track => "Track",
        LoopStatus::Playlist => "Playlist",
    }
}

fn join_option_string(list: Option<Vec<&str>>) -> Option<String> {
    list.map(|a| a.join(", "))
}

impl<'a> MetadataView<'a> {
    pub(crate) fn from_player(
        metadata: &'a Metadata,
        player: &'a Player,
    ) -> Result<MetadataView<'a>, DBusError> {
        let playback_status = player.get_playback_status()?;
        let playback_status_str = playback_status_str(playback_status);

        let loop_status = player.get_loop_status()?;
        let loop_status_str = loop_status_str(loop_status);

        let position = player.get_position()?;
        let position_in_microseconds =
            position.as_secs() + position.subsec_nanos() as u64 * 1_000_000_000;
        let position_in_seconds = position.as_secs();

        let playback_rate = player.get_playback_rate()?;
        let shuffled = player.get_shuffle()?;

        Ok(MetadataView {
            album_artists: metadata.album_artists(),
            album_artists_string: join_option_string(metadata.album_artists()),
            album_name: metadata.album_name(),
            art_url: metadata.art_url(),
            artists: metadata.artists(),
            artists_string: join_option_string(metadata.artists()),
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
            track_id: metadata.track_id().map(TrackID::into),
            track_number: metadata.track_number(),
            url: metadata.url(),
            volume: player.get_volume()?,

            is_looping_playlist: loop_status == LoopStatus::Playlist,
            is_looping_track: loop_status == LoopStatus::Track,
            is_playing: playback_status == PlaybackStatus::Playing,
            is_shuffled: shuffled,
            is_paused: playback_status == PlaybackStatus::Paused,
            is_stopped: playback_status == PlaybackStatus::Stopped,

            raw: HashMap::new(), // TODO: mpris does not allow us to read this without consuming the entire Metadata.
        })
    }

    pub(crate) fn from_progress(progress: &'a Progress) -> Result<MetadataView<'a>, DBusError> {
        let playback_status = progress.playback_status();
        let playback_status_str = playback_status_str(playback_status);

        let loop_status = progress.loop_status();
        let loop_status_str = loop_status_str(loop_status);

        let position = progress.position();
        let position_in_microseconds =
            position.as_secs() + position.subsec_nanos() as u64 * 1_000_000_000;
        let position_in_seconds = position.as_secs();

        let metadata = progress.metadata();

        Ok(MetadataView {
            album_artists: metadata.album_artists(),
            album_artists_string: join_option_string(metadata.album_artists()),
            album_name: metadata.album_name(),
            art_url: metadata.art_url(),
            artists: metadata.artists(),
            artists_string: join_option_string(metadata.artists()),
            auto_rating: metadata.auto_rating(),
            disc_number: metadata.disc_number(),
            length_in_microseconds: metadata.length_in_microseconds(),
            length_in_seconds: metadata.length_in_microseconds().map(|us| us / 1000 / 1000),
            loop_status: loop_status_str,
            playback_rate: 1.0, // TODO
            playback_status: playback_status_str,
            position_in_microseconds,
            position_in_seconds,
            title: metadata.title(),
            track_id: metadata.track_id().map(TrackID::into),
            track_number: metadata.track_number(),
            url: metadata.url(),
            volume: progress.current_volume(),

            is_looping_playlist: loop_status == LoopStatus::Playlist,
            is_looping_track: loop_status == LoopStatus::Track,
            is_playing: playback_status == PlaybackStatus::Playing,
            is_shuffled: progress.shuffle(),
            is_paused: playback_status == PlaybackStatus::Paused,
            is_stopped: playback_status == PlaybackStatus::Stopped,

            raw: HashMap::new(), // TODO: mpris does not allow us to read this without consuming the entire Metadata.
        })
    }
}

pub(crate) fn run(options: &Options, settings: &Settings) -> Result<(), Error> {
    let player = settings.find_player()?;
    let metadata = player.get_metadata()?;
    let metadata_view = MetadataView::from_player(&metadata, &player)?;

    match options.format {
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
    print_text_field("Track ID", &view.track_id);
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
