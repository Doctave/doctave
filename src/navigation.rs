use serde::Serialize;
use std::path::{Path, PathBuf};

pub fn build(documents: &[(PathBuf, String)]) -> Level {
    build_level(&documents, Path::new("")).expect("No root level found")
}

pub fn build_level(documents: &[(PathBuf, String)], level: &Path) -> Option<Level> {
    let mut links = vec![];
    let mut child_levels = vec![];

    for (path, title) in documents {
        match path.parent() {
            Some(p) => {
                // If this path belongs to the current level,
                // add it to the list of links
                if p == level {
                    links.push(Link {
                        path: path.clone(),
                        title: title.clone(),
                    });
                } else {
                    // If the path is one level higher above this level, add it
                    // to my child levels
                    println!("Parent: {:?} , Level: {:?}", p, level);
                    if p.parent() == Some(level) {
                        println!("Adding child level {:?} to {:?}", p, level);
                        child_levels.push(p)
                    }
                }
            }
            _ => {}
        }
    }

    child_levels.sort();
    child_levels.dedup();

    let mut children = vec![];

    for level in child_levels {
        if let Some(l) = build_level(&documents, &Path::new(level)) {
            children.push(l);
        }
    }

    if links.len() > 0 {
        Some(Level { links, children })
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Level {
    links: Vec<Link>,
    children: Vec<Level>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct Link {
    path: PathBuf,
    title: String,
}

#[cfg(test)]
mod test {
    use super::*;

    fn page(path: &str, name: &str) -> (PathBuf, String) {
        (PathBuf::from(path), name.to_string())
    }

    #[test]
    fn basic() {
        let paths = vec![
            page("index.html", "Getting Started"),
            page("sibling/index.html", "Child index"),
            page("sibling/child.html", "Child sibling"),
        ];

        assert_eq!(
            build(&paths),
            Level {
                links: vec![Link {
                    path: PathBuf::from("index.html"),
                    title: "Getting Started".to_string(),
                }],
                children: vec![Level {
                    links: vec![
                        Link {
                            path: PathBuf::from("sibling/index.html"),
                            title: "Child index".to_string()
                        },
                        Link {
                            path: PathBuf::from("sibling/child.html"),
                            title: "Child sibling".to_string(),
                        }
                    ],
                    children: vec![]
                }]
            }
        )
    }
}
