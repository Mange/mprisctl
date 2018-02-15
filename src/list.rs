use super::Settings;

pub(crate) fn run(settings: &Settings) {
    eprintln!(
        "This should be a list, but that's not done yet. At least you can see your settings:\n{:#?}",
        settings,
    );
}
