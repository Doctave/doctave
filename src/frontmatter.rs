use std::collections::BTreeMap;

pub fn parse(input: &str) -> std::io::Result<BTreeMap<String, String>> {
    if input.starts_with("---\n") {
        let after_starter_mark = &input[4..];
        let end_mark = after_starter_mark.find("---\n");

        if end_mark.is_none() {
            return Ok(BTreeMap::new());
        };

        serde_yaml::from_str(&input[4..end_mark.unwrap() + 4])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    } else {
        Ok(BTreeMap::new())
    }
}

pub fn end_pos(input: &str) -> usize {
    if input.starts_with("---\n") {
        let after_starter_mark = &input[4..];
        let end_mark = after_starter_mark.find("---\n");

        if end_mark.is_none() {
            0
        } else {
            end_mark.unwrap() + 4
        }
    } else {
        0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_parse() {
        let input = indoc! {"
            ---
            title: Runbooks
            ---

            # Runbooks
        "};

        let values = parse(input).unwrap();

        let expected = Some("Runbooks".to_owned());
        let actual = values.get("title");

        assert_eq!(actual, expected.as_ref());
    }

    #[test]
    fn missing_frontmatter() {
        let input = indoc! {"
            # Some content
        "};

        let values = parse(input).unwrap();

        assert_eq!(values, BTreeMap::new());
    }

    #[test]
    fn invalid_yaml() {
        let input = indoc! {"
            ---
            :::blarg: @@!~
            ---

            # Some content
        "};

        assert!(parse(input).is_err());
    }

    #[test]
    fn never_ending_frontmatter() {
        let input = indoc! {"
            ---
            title: Runbooks

            # Runbooks
        "};

        assert_eq!(parse(input).unwrap(), BTreeMap::new());
    }
}
