extern crate handlebars;

mod join;
mod or;
mod time;

use clap::ArgMatches;
use super::{Error, Settings};
use self::handlebars::{no_escape, Handlebars, RenderError, TemplateError, TemplateRenderError};
use metadata::MetadataView;
use std::borrow::Cow;
use mpris::Player;

pub const DEFAULT_INTERVAL_MS: u32 = 250;
pub const DEFAULT_INTERVAL_MS_STR: &str = "250";

pub(crate) fn run(matches: Option<&ArgMatches>, settings: &Settings) -> Result<(), Error> {
    let matches = matches.unwrap();
    let template = matches.value_of("format").unwrap(); // Field marked as required in arglist
    let watch = matches.is_present("watch");

    let player = settings.find_player()?;

    let handlebars = setup_handlebars(&template)?;

    if watch {
        let watch_interval = matches
            .value_of("watch-interval")
            .unwrap()
            .parse()
            .unwrap_or(DEFAULT_INTERVAL_MS);
        watch_player(player, handlebars, watch_interval)?
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

impl From<TemplateRenderError> for Error {
    fn from(template_error: TemplateRenderError) -> Error {
        match template_error {
            TemplateRenderError::TemplateError(error) => error.into(),
            TemplateRenderError::RenderError(error) => error.into(),
            TemplateRenderError::IOError(error, _) => error.into(),
        }
    }
}

impl From<TemplateError> for Error {
    fn from(template_error: TemplateError) -> Error {
        Error::TemplateError(format!(
            "{message}\n(at line {line}, column {column})",
            message = template_error.reason,
            line = option_usize_to_string(template_error.line_no),
            column = option_usize_to_string(template_error.column_no)
        ))
    }
}

impl From<RenderError> for Error {
    fn from(render_error: RenderError) -> Error {
        Error::RenderError(format!(
            "{message}\n(at line {line}, column {column})",
            message = render_error.desc,
            line = option_usize_to_string(render_error.line_no),
            column = option_usize_to_string(render_error.column_no)
        ))
    }
}

fn option_usize_to_string<'a>(n: Option<usize>) -> Cow<'a, str> {
    match n {
        Some(num) => Cow::Owned(num.to_string()),
        None => Cow::Borrowed("?"),
    }
}
