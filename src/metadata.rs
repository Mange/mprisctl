extern crate mpris;
extern crate serde_json;

use std::fmt::Display;
use super::{Error, Settings};
use clap::ArgMatches;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Format {
    Text,
    JSON,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MetadataView<'a> {
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
    title: Option<&'a str>,
    track_id: &'a str,
    track_number: Option<i32>,
    url: Option<&'a str>,
    // rest: HashMap<String, MetadataValue>,
}

impl<'a> From<&'a mpris::Metadata> for MetadataView<'a> {
    fn from(metadata: &'a mpris::Metadata) -> MetadataView<'a> {
        MetadataView {
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
            title: metadata.title(),
            track_id: metadata.track_id(),
            track_number: metadata.track_number(),
            url: metadata.url(),
        }
    }
}

pub(crate) fn run(matches: Option<&ArgMatches>, settings: &Settings) -> Result<(), Error> {
    let metadata = settings.find_player()?.get_metadata()?;
    let metadata_view = MetadataView::from(&metadata);

    let format = match matches {
        Some(matches) => if matches.is_present("json") {
            Format::JSON
        } else {
            Format::Text
        },
        None => Format::Text,
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
    print_text_field("Track ID", &Some(view.track_id));
    print_text_field("Title", &view.title);
    print_text_field("Artists", &view.artists_string);
    print_text_field("Album", &view.album_name);
    print_text_field("Track number", &view.track_number);
    print_text_field("Album artists", &view.album_artists_string);
    print_text_field("Artwork URL", &view.art_url);
    print_text_field("Auto-rating", &view.auto_rating);
    print_text_field("Disc number", &view.disc_number);
    print_text_field("Length (µs)", &view.length_in_microseconds);
    print_text_field("Length (s)", &view.length_in_seconds);
    print_text_field("URL", &view.url);
    Ok(())
}

// Length of longest text field text ("Album artists")
const TEXT_FIELD_PADDING: usize = 13;

fn print_text_field<'a, T: Display>(title: &'static str, value: &Option<T>) {
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
