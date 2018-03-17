extern crate handlebars;

use clap::ArgMatches;
use super::{Error, Settings};
use self::handlebars::{Handlebars, RenderError, TemplateError, TemplateRenderError};
use metadata::MetadataView;
use std::borrow::Cow;

pub(crate) fn run(matches: Option<&ArgMatches>, settings: &Settings) -> Result<(), Error> {
    let matches = matches.unwrap();
    let template = matches.value_of("format").unwrap(); // Field marked as required in arglist

    let player = settings.find_player()?;
    let metadata = player.get_metadata()?;
    let metadata_view = MetadataView::from_player(&metadata, &player)?;

    println!("{}", render_template(&template, &metadata_view)?);
    Ok(())
}

fn render_template(template: &str, metadata_view: &MetadataView) -> Result<String, Error> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    handlebars
        .render_template(template, metadata_view)
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
