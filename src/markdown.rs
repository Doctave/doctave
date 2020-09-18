use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use serde::Serialize;

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

    let parser = Parser::new_ext(input, options).filter_map(|event| match event {
        // Mermaid JS code block tranformations
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(inner))) => {
            let lang = inner.split(' ').next().unwrap();
            if lang == "mermaid" {
                Some(Event::Html(CowStr::Borrowed("<div class=\"mermaid\">")))
            } else {
                Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(inner))))
            }
        }
        Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(inner))) => {
            let lang = inner.split(' ').next().unwrap();
            if lang == "mermaid" {
                Some(Event::Html(CowStr::Borrowed("</div>")))
            } else {
                Some(Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(inner))))
            }
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
            }
            Some(Event::Text(text))
        }
        _ => Some(event),
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
