extern crate mpris;
use mpris::Player;

use super::{Error, Settings, Verbosity};

pub(crate) fn run(settings: &Settings) -> Result<(), Error> {
    let player = settings.find_player()?;

    if player.checked_play()? {
        command_completed(&settings.verbosity, &player);
        Ok(())
    } else {
        command_failed(&settings.verbosity, &player);
        Ok(())
    }
}

fn command_completed(verbosity: &Verbosity, player: &Player) {
    match *verbosity {
        Verbosity::Quiet | Verbosity::Normal => {}
        Verbosity::Verbose => {
            eprintln!("Play command sent to {}", player.identity());
        }
    }
}

fn command_failed(verbosity: &Verbosity, player: &Player) {
    match *verbosity {
        Verbosity::Quiet => {}
        Verbosity::Normal | Verbosity::Verbose => {
            eprintln!(
                "Play command not sent to {} as player does not accept it.",
                player.identity()
            );
        }
    }
}
