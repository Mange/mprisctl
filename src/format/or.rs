use super::handlebars::*;

pub(crate) fn helper(
    h: &Helper,
    registry: &Handlebars,
    rc: &mut RenderContext,
    out: &mut Output,
) -> HelperResult {
    let first_value = h.params()
        .iter()
        .map(|param| param.value())
        .find(|value| !value.is_null());
    if let Some(value) = first_value {
        out.write(value.render().as_ref())?;
    } else {
        if let Some(template) = h.template() {
            template.render(registry, rc, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renders_first_available_value() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("or", Box::new(helper));

        assert_eq!(
            handlebars
                .render_template(r#"{{or "first" null}}"#, &())
                .unwrap(),
            "first"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{or null null "third"}}"#, &())
                .unwrap(),
            "third"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{or false "string"}}"#, &())
                .unwrap(),
            "false"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{or null ["string"]}}"#, &())
                .unwrap(),
            "[string, ]"
        );
    }

    #[test]
    fn it_renders_nested_when_no_value_is_present() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("or", Box::new(helper));

        assert_eq!(
            handlebars
                .render_template(r#"{{#or true}}this is the fallback{{/or}}"#, &())
                .unwrap(),
            "true"
        );

        assert_eq!(
            handlebars
                .render_template(r#"{{#or null}}this is the fallback{{/or}}"#, &())
                .unwrap(),
            "this is the fallback"
        );
    }
}
