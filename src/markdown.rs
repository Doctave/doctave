use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use serde::Serialize;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

#[derive(Debug, PartialEq, Clone)]
pub struct Markdown {
    pub as_html: String,
    pub headings: Vec<Heading>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Heading {
    pub title: String,
    pub anchor: String,
    pub level: u16,
}

pub fn parse(input: &str) -> Markdown {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);

    let mut headings = vec![];
    let mut heading_level = 0;
    let mut heading_index = 1u32;
    let mut codeblock_language = None;

    let parser = Parser::new_ext(input, options).filter_map(|event| {
        match event {
            // Mermaid JS code block tranformations
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(inner))) => {
                let lang = inner.split(' ').next().unwrap();

                if lang == "mermaid" {
                    Some(Event::Html(CowStr::Borrowed("<div class=\"mermaid\">")))
                } else {
                    codeblock_language = Some(
                        SYNTAX_SET
                            .find_syntax_by_token(lang)
                            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text()),
                    );
                    Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(inner))))
                }
            }
            e @ Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                codeblock_language = Some(SYNTAX_SET.find_syntax_plain_text());
                Some(e)
            }
            Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(inner))) => {
                codeblock_language = None;

                let lang = inner.split(' ').next().unwrap();
                if lang == "mermaid" {
                    Some(Event::Html(CowStr::Borrowed("</div>")))
                } else {
                    Some(Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(inner))))
                }
            }
            e @ Event::End(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                codeblock_language = None;
                Some(e)
            }

            // Apply heading anchor tags
            Event::Start(Tag::Heading(level @ 1..=6)) => {
                heading_level = level;
                None
            }
            Event::Text(text) => {
                if heading_level != 0 {
                    let mut anchor = text
                        .clone()
                        .into_string()
                        .trim()
                        .to_lowercase()
                        .replace(" ", "-");

                    anchor.push('-');
                    anchor.push_str(&heading_index.to_string());

                    let tmp = Event::Html(CowStr::from(format!(
                        "<h{} id=\"{}\">{}",
                        heading_level, anchor, text
                    )))
                    .into();

                    heading_index += 1;
                    headings.push(Heading {
                        anchor,
                        title: text.to_string(),
                        level: heading_level as u16,
                    });

                    heading_level = 0;
                    return tmp;
                } else if let Some(lang) = codeblock_language {
                    let mut h =
                        HighlightLines::new(lang, &THEME_SET.themes["InspiredGitHub"]);
                    let regions = h.highlight(&text, &SYNTAX_SET);
                    let html = styled_line_to_highlighted_html(&regions[..], IncludeBackground::No);

                    Some(Event::Html(html.into()))
                } else {
                    Some(Event::Text(text))
                }
            }
            _ => Some(event),
        }
    });

    // Write to String buffer.
    let mut as_html = String::new();
    html::push_html(&mut as_html, parser);

    Markdown { as_html, headings }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_markdown_doc() {
        let input = indoc! {"
        # My heading

        Some content

        ## Some other heading
        "};

        let Markdown { as_html, headings } = parse(&input);

        assert_eq!(
            as_html,
            indoc! {"
                <h1 id=\"my-heading-1\">My heading</h1>
                <p>Some content</p>
                <h2 id=\"some-other-heading-2\">Some other heading</h2>
            "}
        );

        assert_eq!(
            headings,
            vec![
                Heading {
                    title: "My heading".to_string(),
                    anchor: "my-heading-1".to_string(),
                    level: 1,
                },
                Heading {
                    title: "Some other heading".to_string(),
                    anchor: "some-other-heading-2".to_string(),
                    level: 2,
                }
            ]
        );
    }
}
