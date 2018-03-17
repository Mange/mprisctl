use super::handlebars::*;
use serde_json::Value;

pub(crate) fn helper(
    h: &Helper,
    _: &Handlebars,
    _: &mut RenderContext,
    out: &mut Output,
) -> HelperResult {
    if let Some(joining) = h.param(0) {
        let joiner = joining.value().render();

        let mut iter = h.params()[1..]
            .iter()
            .map(|param| param.value())
            .flat_map(|value| match value {
                &Value::Array(ref array) => array.clone(),
                _ => vec![value.clone()],
            })
            .filter(|value| !value.is_null())
            .peekable();

        loop {
            let next = iter.next();
            let peek = iter.peek();

            match (next, peek) {
                (Some(next), Some(_)) => {
                    out.write(next.render().as_ref())?;
                    out.write(&joiner)?;
                }
                (Some(next), None) => {
                    out.write(next.render().as_ref())?;
                }
                (None, _) => break,
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_joins_arrays() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("join", Box::new(helper));

        let values = json!({
            "letters": ["a", "b", "c"],
            "numbers": [1, 2, 3,],
            "holes": [1, null, 3],
        });

        assert_eq!(
            handlebars
                .render_template(r#"{{join "+" numbers}}"#, &values)
                .unwrap(),
            "1+2+3"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{join "+" numbers holes}}"#, &values)
                .unwrap(),
            "1+2+3+1+3"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{join "." letters "and so on"}}"#, &values)
                .unwrap(),
            "a.b.c.and so on"
        );
    }

    #[test]
    fn it_renders_nothing_on_empty_values() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("join", Box::new(helper));

        assert_eq!(
            handlebars
                .render_template(r#"{{join "." []}}"#, &())
                .unwrap(),
            ""
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{join "." [null, null, null]}}"#, &())
                .unwrap(),
            ""
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{join "." [1, null, null]}}"#, &())
                .unwrap(),
            "1"
        );
    }

    #[test]
    fn it_renders_nothing_on_missing_arguments() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("join", Box::new(helper));

        assert_eq!(
            handlebars.render_template(r#"{{join "."}}"#, &()).unwrap(),
            ""
        );

        assert_eq!(handlebars.render_template(r#"{{join}}"#, &()).unwrap(), "");
    }
}
