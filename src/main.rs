extern crate caseless;
extern crate mpris;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

mod list;
mod basic_command;
mod metadata;
mod format;

use list::run as list;
use basic_command::run as basic_command;
use metadata::run as metadata;
use format::run as format;

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

#[derive(Debug, Default)]
struct Settings {
    pub verbosity: Verbosity,
    pub player_selection: PlayerSelection,
}

impl<'a> From<&'a clap::ArgMatches<'a>> for Settings {
    fn from(matches: &'a clap::ArgMatches) -> Settings {
        let verbosity = if matches.is_present("quiet") {
            Verbosity::Quiet
        } else if matches.is_present("verbose") {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        };

        let player_selection = if let Some(player_name) = matches.value_of("player") {
            PlayerSelection::WithName(String::from(player_name))
        } else {
            PlayerSelection::Automatic
        };

        Settings {
            player_selection: player_selection,
            verbosity: verbosity,
        }
    }
}

impl Settings {
    fn find_player<'p>(&self) -> Result<Player<'p>, Error> {
        use mpris::FindingError;
        let finder = PlayerFinder::new()?;

        match self.player_selection {
            PlayerSelection::Automatic => match finder.find_active() {
                Ok(player) => Ok(player),
                Err(FindingError::DBusError(err)) => Err(err.into()),
                Err(FindingError::NoPlayerFound) => Err(Error::AutomaticPlayerNotFound),
            },
            PlayerSelection::WithName(ref name) => match finder.find_all() {
                Ok(players) => find_player_with_name(players, name),
                Err(FindingError::DBusError(err)) => Err(err.into()),
                Err(FindingError::NoPlayerFound) => Err(Error::NamedPlayerNotFound(name.clone())),
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
        None => Err(Error::NamedPlayerNotFound(String::from(name))),
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)] DBusError(mpris::DBusError),
    #[fail(display = "Could not find any player")] AutomaticPlayerNotFound,
    #[fail(display = "Could not find any player with name \"{}\"", _0)] NamedPlayerNotFound(String),
    #[fail(display = "Could not convert to JSON: {}", _0)]
    JsonSerializationError(serde_json::Error),
    #[fail(display = "I/O error: {}", _0)] IOError(std::io::Error),
    #[fail(display = "Template error: {}", _0)] TemplateError(String),
    #[fail(display = "Could not render template: {}", _0)] RenderError(String),
}

impl From<mpris::DBusError> for Error {
    fn from(dbus_error: mpris::DBusError) -> Error {
        Error::DBusError(dbus_error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(serde_error: serde_json::Error) -> Error {
        Error::JsonSerializationError(serde_error)
    }
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Error {
        Error::IOError(io_error)
    }
}

fn build_app<'a, 'b>() -> App<'a, 'b> {
    app_from_crate!()
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::GlobalVersion)
        .global_setting(AppSettings::InferSubcommands)
        .global_setting(AppSettings::VersionlessSubcommands)
        .global_setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("Turns on verbose output")
                .overrides_with("quiet")
                .global(true),
        )
        .arg(
            Arg::with_name("player")
                .long("player")
                .short("p")
                .value_name("NAME")
                .help("Tries to control player with given name")
                .global(true),
        )
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("Output as little as possible")
                .global(true),
        )
        .subcommand(SubCommand::with_name("list").about("List running players"))
        .subcommand(SubCommand::with_name("play").about("Resume current media"))
        .subcommand(SubCommand::with_name("pause").about("Pause current media"))
        .subcommand(
            SubCommand::with_name("toggle-pause").about("Pause if playing, or play if paused"),
        )
        .subcommand(SubCommand::with_name("next").about("Skip to next media"))
        .subcommand(SubCommand::with_name("previous").about("Go back to previous media"))
        .subcommand(
            SubCommand::with_name("metadata")
                .about("Print metadata about the current media")
                .arg(
                    Arg::with_name("text")
                        .long("text")
                        .help("Print metadata in text format (default)"),
                )
                .arg(
                    Arg::with_name("json")
                        .long("json")
                        .help("Print metadata as JSON")
                        .overrides_with("text"),
                ),
        )
        .subcommand(
            SubCommand::with_name("format")
                .about("Custom format of player metadata")
                .arg(
                    Arg::with_name("format")
                        .required(true)
                        .help("Format string")
                        .long_help(include_str!("format_help.txt")),
                ),
        )
}

fn main() {
    let app = build_app();
    let matches = app.get_matches();
    let settings = Settings::from(&matches);

    let result = match matches.subcommand() {
        ("list", _) => list(&settings),
        ("play", _) => basic_command("Play", Player::checked_play, &settings),
        ("pause", _) => basic_command("Pause", Player::checked_pause, &settings),
        ("toggle-pause", _) => basic_command("Play/Pause", Player::checked_play_pause, &settings),
        ("next", _) => basic_command("Next", Player::checked_next, &settings),
        ("previous", _) => basic_command("Previous", Player::checked_previous, &settings),
        ("metadata", matches) => metadata(matches, &settings),
        ("format", matches) => format(matches, &settings),
        (unknown, _) => panic!("Software bug: No subcommand is implemented for {}", unknown),
    };

    match result {
        Ok(()) => { /* :-) */ }
        Err(error) => {
            eprintln!("{}", error);
            ::std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod settings {
        use super::*;

        fn settings_from(args: Vec<&'static str>) -> Settings {
            Settings::from(&build_app().get_matches_from(args))
        }

        #[test]
        fn it_sets_verbosity() {
            let settings = settings_from(vec!["x", "play"]);
            assert_eq!(settings.verbosity, Verbosity::Normal);

            let settings = settings_from(vec!["x", "play", "-v"]);
            assert_eq!(settings.verbosity, Verbosity::Verbose);

            let settings = settings_from(vec!["x", "play", "--quiet"]);
            assert_eq!(settings.verbosity, Verbosity::Quiet);

            let settings = settings_from(vec!["x", "--verbose", "play", "-q"]);
            assert_eq!(settings.verbosity, Verbosity::Quiet);
        }

        #[test]
        fn it_sets_player_selection() {
            let settings = settings_from(vec!["x", "play"]);
            assert_eq!(settings.player_selection, PlayerSelection::Automatic);

            let settings = settings_from(vec!["x", "-p", "vlc", "play"]);
            assert_eq!(
                settings.player_selection,
                PlayerSelection::WithName(String::from("vlc"))
            );

            let settings = settings_from(vec!["x", "play", "-p", "spotify"]);
            assert_eq!(
                settings.player_selection,
                PlayerSelection::WithName(String::from("spotify"))
            );
        }
    }
}
