extern crate mpris;
use mpris::Player;

use super::{Error, Settings, Verbosity};

pub(crate) fn run<'p, F>(name: &'static str, action: F, settings: &Settings) -> Result<(), Error>
where
    F: FnOnce(&Player<'p>) -> Result<bool, mpris::DBusError>,
{
    let player = settings.find_player()?;

    if action(&player)? {
        command_sent(name, &settings.verbosity, &player);
        Ok(())
    } else {
        command_not_supported(name, &settings.verbosity, &player);
        Ok(())
    }
}

fn command_sent(name: &'static str, verbosity: &Verbosity, player: &Player) {
    match *verbosity {
        Verbosity::Quiet | Verbosity::Normal => {}
        Verbosity::Verbose => {
            eprintln!("{} command sent to {}", name, player.identity());
        }
    }
}

fn command_not_supported(name: &'static str, verbosity: &Verbosity, player: &Player) {
    match *verbosity {
        Verbosity::Quiet => {}
        Verbosity::Normal | Verbosity::Verbose => {
            eprintln!(
                "{} command not sent to {} as player does not accept it.",
                name,
                player.identity()
            );
        }
    }
}
