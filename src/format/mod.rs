extern crate handlebars;

mod join;
mod or;
mod time;

use structopt::StructOpt;
use self::handlebars::{no_escape, Handlebars};
use super::Settings;
use failure::Error;
use metadata::MetadataView;
use mpris::Player;

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(short = "w", long = "watch")]
    /// Keep running, outputting the template every time the rendered template is different because
    /// of metadata changes in the player.
    watch: bool,

    #[structopt(short = "i", long = "watch-interval", value_name = "MILLISECONDS", default_value = "250")]
    /// Rerender at close to this interval when watching. Shorter time means quicker updates, while
    /// longer time means less resource utilization.
    watch_interval: u32,

    #[structopt(name = "FORMAT", raw(long_help = "include_str!(\"../format_help.txt\")"))]
    /// The format string. Full reference is available under the --help option.
    template: String,
}

pub(crate) fn run(options: &Options, settings: &Settings) -> Result<(), Error> {
    let player = settings.find_player()?;

    let handlebars = setup_handlebars(&options.template)?;

    if options.watch {
        watch_player(player, handlebars, options.watch_interval)?
    } else {
        let metadata = player.get_metadata()?;
        let metadata_view = MetadataView::from_player(&metadata, &player)?;
        println!("{}", render_template(&handlebars, &metadata_view)?);
    }
    Ok(())
}

fn setup_handlebars(template: &str) -> Result<Handlebars, Error> {
    let mut handlebars = Handlebars::new();

    handlebars.set_strict_mode(false);
    handlebars.register_escape_fn(no_escape);
    handlebars.register_helper("join", Box::new(join::helper));
    handlebars.register_helper("or", Box::new(or::helper));
    handlebars.register_helper("time", Box::new(time::helper));

    if let Err(error) = handlebars.register_template_string("main", template) {
        return Err(error.into());
    }

    Ok(handlebars)
}

fn watch_player(player: Player, handlebars: Handlebars, interval: u32) -> Result<(), Error> {
    let mut tracker = player.track_progress(interval)?;
    let mut last_output = String::new();
    loop {
        let (progress, _) = tracker.tick();
        let metadata_view = MetadataView::from_progress(progress)?;
        let output = render_template(&handlebars, &metadata_view)?;
        if output != last_output {
            println!("{}", output);
            last_output = output;
        }
    }
}

fn render_template(handlebars: &Handlebars, metadata_view: &MetadataView) -> Result<String, Error> {
    handlebars
        .render("main", metadata_view)
        .map_err(|e| e.into())
}
