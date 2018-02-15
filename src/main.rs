extern crate mpris;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

mod list;
use list::run as list;

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
    verbosity: Verbosity,
    player_selection: PlayerSelection,
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

fn build_app<'a, 'b>() -> App<'a, 'b> {
    app_from_crate!()
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::InferSubcommands)
        .setting(AppSettings::VersionlessSubcommands)
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
            SubCommand::with_name("toggle_pause").about("Pause if playing, or play if paused."),
        )
        .subcommand(SubCommand::with_name("next").about("Skip to next media"))
        .subcommand(SubCommand::with_name("previous").about("Go back to previous media"))
}

fn main() {
    let app = build_app();
    let matches = app.get_matches();
    let settings = Settings::from(&matches);

    match matches.subcommand() {
        ("list", _) => list(&settings),
        ("play", _) => unimplemented!("play is not implemented yet"),
        ("pause", _) => unimplemented!("pause is not implemented yet"),
        ("toggle_pause", _) => unimplemented!("toggle_pause is not implemented yet"),
        ("next", _) => unimplemented!("next is not implemented yet"),
        ("previous", _) => unimplemented!("previous is not implemented yet"),
        (unknown, _) => panic!("Software bug: No subcommand is implemented for {}", unknown),
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
