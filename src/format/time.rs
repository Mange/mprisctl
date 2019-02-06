use super::handlebars::*;
use serde_json::Value;

pub(crate) fn helper(
    h: &Helper,
    _: &Handlebars,
    _: &mut RenderContext,
    out: &mut Output,
) -> HelperResult {
    if let Some(time) = h.param(0).map(|p| p.value()) {
        // What width do the user want?
        let width = match h.param(1) {
            Some(param) => width_of_value(param.value()),
            None => width_of_value(time),
        };

        if let Some(seconds) = time.as_u64() {
            return render_time(seconds, width, out);
        }
    }
    Ok(())
}

use std::u64::MAX as u64_max;
const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;

enum Width {
    Minute,
    Hour,
    Invalid,
}

fn width_of_value(val: &Value) -> Width {
    match val {
        &Value::Number(ref num) => match num.as_u64() {
            Some(0...HOUR) => Width::Minute,
            Some(HOUR...u64_max) => Width::Hour,
            _ => Width::Invalid,
        },
        &Value::String(ref s) if s == "hour" => Width::Hour,
        &Value::String(ref s) if s == "minute" => Width::Minute,
        _ => Width::Invalid,
    }
}

fn render_time(seconds: u64, width: Width, out: &mut Output) -> HelperResult {
    let mut seconds = seconds;

    let whole_hours = seconds / HOUR;
    seconds -= whole_hours * HOUR;

    let whole_minutes = seconds / MINUTE;
    seconds -= whole_minutes * MINUTE;

    match width {
        Width::Hour => out.write(&format!(
            "{:02}:{:02}:{:02}",
            whole_hours, whole_minutes, seconds
        ))?,
        Width::Minute | Width::Invalid => {
            out.write(&format!("{:02}:{:02}", whole_minutes, seconds))?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_renders_with_dynamic_width() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("time", Box::new(helper));

        let values = json!({
            "long": (2 * 60 * 60) + (5 * 60) + 34,
            "middle": (5 * 60) + 34,
            "short": 34,
        });

        assert_eq!(
            handlebars
                .render_template(r#"{{time long}}"#, &values)
                .unwrap(),
            "02:05:34"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{time middle}}"#, &values)
                .unwrap(),
            "05:34"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{time short}}"#, &values)
                .unwrap(),
            "00:34"
        );
    }

    #[test]
    fn it_renders_with_set_width() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("time", Box::new(helper));

        let values = json!({
            "long": (2 * 60 * 60) + (5 * 60) + 34,
            "middle": (5 * 60) + 34,
            "short": 34,
        });

        assert_eq!(
            handlebars
                .render_template(r#"{{time long "hour"}}"#, &values)
                .unwrap(),
            "02:05:34"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{time short "hour"}}"#, &values)
                .unwrap(),
            "00:00:34"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{time long "minute"}}"#, &values)
                .unwrap(),
            "05:34"
        );
    }

    #[test]
    fn it_renders_with_width_from_other_value() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("time", Box::new(helper));

        let values = json!({
            "long": (2 * 60 * 60) + (5 * 60) + 34,
            "middle": (5 * 60) + 34,
            "short": 34,
        });

        assert_eq!(
            handlebars
                .render_template(r#"{{time long short}}"#, &values)
                .unwrap(),
            "05:34"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{time middle long}}"#, &values)
                .unwrap(),
            "00:05:34"
        );
    }

    #[test]
    fn it_falls_back_on_dynamic_width_on_bad_width() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("time", Box::new(helper));

        let values = json!({
            "long": (2 * 60 * 60) + (5 * 60) + 34,
            "middle": (5 * 60) + 34,
            "short": 34,
        });

        assert_eq!(
            handlebars
                .render_template(r#"{{time middle "bad"}}"#, &values)
                .unwrap(),
            "05:34"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{time middle null}}"#, &values)
                .unwrap(),
            "05:34"
        );
    }

    #[test]
    fn it_renders_nothing_when_value_is_bad() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("time", Box::new(helper));

        assert_eq!(
            handlebars
                .render_template(r#"{{time "bad"}}"#, &())
                .unwrap(),
            ""
        );

        assert_eq!(
            handlebars.render_template(r#"{{time -100}}"#, &()).unwrap(),
            ""
        );

        assert_eq!(
            handlebars.render_template(r#"{{time NaN}}"#, &()).unwrap(),
            ""
        );
    }
}
