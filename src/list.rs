extern crate mpris;
use mpris::Player;

use super::{Error, Settings, Verbosity};

pub(crate) fn run(settings: &Settings) -> Result<(), Error> {
    let finder = mpris::PlayerFinder::new()?;
    let players = match finder.find_all() {
        Ok(players) => players,
        Err(mpris::FindingError::NoPlayerFound) => vec![],
        Err(mpris::FindingError::DBusError(err)) => {
            return Err(Error::from(err));
        }
    };

    if players.is_empty() {
        match settings.verbosity {
            Verbosity::Normal | Verbosity::Quiet => {}
            Verbosity::Verbose => {
                eprintln!("No players found.");
            }
        }
        return Ok(());
    }

    if settings.verbosity == Verbosity::Verbose {
        eprintln!("Found players:");
    }

    for player in players.iter() {
        match settings.verbosity {
            Verbosity::Normal => {
                println!("{}", player.identity());
            }
            Verbosity::Quiet => {
                println!("{}", player.identity());
            }
            Verbosity::Verbose => {
                println!("{}", verbose_player(player)?);
            }
        }
    }
    Ok(())
}

fn verbose_player(player: &Player) -> Result<String, Error> {
    use mpris::PlaybackStatus;
    use std::borrow::Cow;

    let playback_status = player.get_playback_status()?;
    let metadata = player.get_metadata()?;

    let title = metadata.title().unwrap_or("Unknown title");
    let artist: Cow<str> = metadata
        .artists()
        .map(|artists| Cow::Owned(artists.join(", ")))
        .unwrap_or(Cow::Borrowed("Unknown artist"));

    match playback_status {
        PlaybackStatus::Playing => Ok(format!(
            "{identity}\t- Playing {title} by {artist}",
            identity = player.identity(),
            title = title,
            artist = artist,
        )),
        PlaybackStatus::Paused => Ok(format!(
            "{identity}\t- Paused on {title} by {artist}",
            identity = player.identity(),
            title = title,
            artist = artist,
        )),
        PlaybackStatus::Stopped => Ok(format!(
            "{identity}\t- Not currently playing anything",
            identity = player.identity(),
        )),
    }
}
