#[cfg(test)]
mod tests {
    use html_template_macros::html;
    use html_template_core::Root;

    #[test]
    fn basic_closure() {
        let capture = 43;
        let dom: Root = html!{
            { capture.to_string() }
        }.into();
        let expected = "43";

        assert_eq!(expected, dom.to_string());
    }

    #[test]
    fn nested_closure() {
        let dom: Root = html!{
            { (0..3).map(|v| html!({[move] format!("{}, ", v)})).collect() }
        }.into();
        let expected = "0, 1, 2, ";

        assert_eq!(expected, dom.to_string());
    }

    #[test]
    fn basic_html() {
        let title = "hello world";
        let dom: Root = html!{
            <html>
                <title>{title.to_string()}</title>
                "wowo mwmw"
            </html>
        }.into();
        let expected = "<html><title>hello world</title>wowo mwmw</html>";

        assert_eq!(expected, dom.to_string());
    }


    #[test]
    fn basic_html_properties() {
        let dom: Root = html!{
            <base href="http://127.0.0.1:8080/" target="_blank">
        }.into();
        let expected = r#"<base href="http://127.0.0.1:8080/" target="_blank">"#;

        assert_eq!(expected, dom.to_string());
    }

    #[test]
    fn html_properties() {
        let value = "value";
        let dom: Root = html!{
            <base href="http://127.0.0.1:8080/" target={format!("\"{}\"", value)}>
        }.into();
        let expected = r#"<base href="http://127.0.0.1:8080/" target="value">"#;

        assert_eq!(expected, dom.to_string());
    }
}
