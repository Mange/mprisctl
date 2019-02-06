extern crate caseless;
extern crate mpris;

#[cfg(test)]
#[macro_use]
extern crate serde_json;

#[cfg(not(test))]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate failure;
use failure::{Error, format_err};

extern crate structopt;
use structopt::StructOpt;

mod basic_command;
mod format;
mod list;
mod metadata;

use basic_command::run as basic_command;
use format::run as format;
use list::run as list;
use metadata::run as metadata;

use mpris::{Player, PlayerFinder};

#[derive(Debug, PartialEq)]
enum Verbosity {
    Verbose,
    Normal,
    Quiet,
}

impl Default for Verbosity {
    fn default() -> Self {
        Verbosity::Normal
    }
}

#[derive(Debug, PartialEq)]
enum PlayerSelection {
    Automatic,
    WithName(String),
}

impl Default for PlayerSelection {
    fn default() -> Self {
        PlayerSelection::Automatic
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    rename_all = "kebab-case",
)]
enum Command {
    /// List running players
    List,

    /// Resume current media
    Play,

    /// Pause current media
    Pause,

    /// Pause if playing, or play if paused
    TogglePause,

    /// Skip to next media
    Next,

    /// Go back to start of media, or go to previous media
    Previous,

    /// Print metadata about the current media
    Metadata(metadata::Options),

    /// Print custom format of metadata about the current media
    Format(format::Options),
}

use structopt::clap::AppSettings;
#[derive(Debug, StructOpt)]
#[structopt(
    raw(
        setting = "AppSettings::SubcommandRequiredElseHelp",
        setting = "AppSettings::GlobalVersion",
        global_settings = "&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands, AppSettings::InferSubcommands]",
    )
)]
struct Settings {
    #[structopt(short = "v", long = "verbose", conflicts_with = "quiet", raw(global = "true"))]
    /// Turns on verbose output.
    pub verbose: bool,

    /// Output as little as possible.
    #[structopt(short = "q", long = "quiet", conflicts_with = "verbose", raw(global = "true"))]
    pub quiet: bool,

    /// Control the player with the given name. If no player is selected then the first player
    /// found will be controlled.
    #[structopt(short = "p", long = "player", value_name = "NAME", raw(global = "true"))]
    pub player_name: Option<String>,

    #[structopt(subcommand)]
    pub command: Command,
}

impl Settings {
    fn verbosity(&self) -> Verbosity {
        if self.quiet {
            Verbosity::Quiet
        } else if self.verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        }
    }

    fn player_selection(&self) -> PlayerSelection {
        self.player_name.as_ref().map(|name| PlayerSelection::WithName(name.to_string())).unwrap_or_default()
    }

    fn find_player<'p>(&self) -> Result<Player<'p>, Error> {
        use mpris::FindingError;
        let finder = PlayerFinder::new()?;

        match self.player_selection() {
            PlayerSelection::Automatic => match finder.find_active() {
                Ok(player) => Ok(player),
                Err(FindingError::DBusError(err)) => Err(err.into()),
                Err(FindingError::NoPlayerFound) => Err(format_err!("Could not find any player")),
            },
            PlayerSelection::WithName(ref name) => match finder.find_all() {
                Ok(players) => find_player_with_name(players, name),
                Err(FindingError::DBusError(err)) => Err(err.into()),
                Err(FindingError::NoPlayerFound) => Err(format_err!("Could not find any player with name \"{}\"", name)),
            },
        }
    }
}

fn find_player_with_name<'a>(players: Vec<Player<'a>>, name: &str) -> Result<Player<'a>, Error> {
    let found_player = players
        .into_iter()
        .find(|player| caseless::default_caseless_match_str(player.identity(), name));

    match found_player {
        Some(player) => Ok(player),
        None => Err(format_err!("Could not find any player with name \"{}\"", name)),
    }
}

fn main() {
    let settings = Settings::from_args();

    let result = match settings.command {
        Command::List => list(&settings),
        Command::Play => basic_command("Play", Player::checked_play, &settings),
        Command::Pause => basic_command("Pause", Player::checked_pause, &settings),
        Command::TogglePause => basic_command("Play/Pause", Player::checked_play_pause, &settings),
        Command::Next => basic_command("Next", Player::checked_next, &settings),
        Command::Previous => basic_command("Previous", Player::checked_previous, &settings),
        Command::Metadata(ref options) => metadata(options, &settings),
        Command::Format(ref options) => format(options, &settings),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        for cause in error.iter_causes() {
            eprintln!("\nCaused by {}", cause);
        }
        ::std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod settings {
        use super::*;

        fn settings_from(args: Vec<&'static str>) -> Settings {
            Settings::from_iter(args.iter())
        }

        #[test]
        fn it_sets_verbosity() {
            let settings = settings_from(vec!["x", "play"]);
            assert_eq!(settings.verbosity(), Verbosity::Normal);

            let settings = settings_from(vec!["x", "play", "-v"]);
            assert_eq!(settings.verbosity(), Verbosity::Verbose);

            let settings = settings_from(vec!["x", "play", "--quiet"]);
            assert_eq!(settings.verbosity(), Verbosity::Quiet);

            let settings = settings_from(vec!["x", "--verbose", "play", "-q"]);
            assert_eq!(settings.verbosity(), Verbosity::Quiet);
        }

        #[test]
        fn it_sets_player_selection() {
            let settings = settings_from(vec!["x", "play"]);
            assert_eq!(settings.player_selection(), PlayerSelection::Automatic);

            let settings = settings_from(vec!["x", "-p", "vlc", "play"]);
            assert_eq!(
                settings.player_selection(),
                PlayerSelection::WithName(String::from("vlc"))
            );

            let settings = settings_from(vec!["x", "play", "-p", "spotify"]);
            assert_eq!(
                settings.player_selection(),
                PlayerSelection::WithName(String::from("spotify"))
            );
        }
    }
}
