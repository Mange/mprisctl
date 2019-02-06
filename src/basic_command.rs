extern crate mpris;
use mpris::Player;

use super::{Error, Settings};

pub(crate) fn run<'p, F>(name: &'static str, action: F, settings: &Settings) -> Result<(), Error>
where
    F: FnOnce(&Player<'p>) -> Result<bool, mpris::DBusError>,
{
    let player = settings.find_player()?;

    if action(&player)? {
        command_sent(name, settings.verbose, &player);
        Ok(())
    } else {
        command_not_supported(name, settings.verbose, &player);
        Ok(())
    }
}

fn command_sent(name: &'static str, is_verbose: bool, player: &Player) {
    if is_verbose {
        eprintln!("{} command sent to {}", name, player.identity());
    }
}

fn command_not_supported(name: &'static str, is_verbose: bool, player: &Player) {
    if is_verbose {
        eprintln!(
            "{} command not sent to {} as player does not accept it.",
            name,
            player.identity()
        );
    }
}
