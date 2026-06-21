use ammonia::Builder;
use pulldown_cmark::{Options, Parser, html};

pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut raw_html = String::new();
    html::push_html(&mut raw_html, parser);

    Builder::default().clean(&raw_html).to_string()
}

#[cfg(test)]
mod tests {
    use super::render_markdown;

    #[test]
    fn markdown_is_rendered_and_sanitized() {
        let rendered = render_markdown("# Hello\n\n<script>alert('x')</script>\n\n**Safe**");

        assert!(rendered.contains("<h1>Hello</h1>"));
        assert!(rendered.contains("<strong>Safe</strong>"));
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("alert"));
    }
}
