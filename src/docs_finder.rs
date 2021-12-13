use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use crate::config::Config;
use crate::{Directory, Document};

use walkdir::WalkDir;

/// Loads the current state of the documentation from disk, returning the root
/// directory which contains all files and nested directories.
pub fn find(config: &Config) -> Directory {
    let mut root_dir = walk_dir(config.docs_dir(), config).unwrap_or(Directory {
        path: config.docs_dir().to_path_buf(),
        docs: vec![],
        dirs: vec![],
    });

    println!("{:#?}", root_dir);

    generate_missing_indices(&mut root_dir, config);

    root_dir
}

fn walk_dir<P: AsRef<Path>>(dir: P, config: &Config) -> Option<Directory> {
    let mut docs = vec![];
    let mut dirs = vec![];

    let current_dir: &Path = dir.as_ref();

    for entry in WalkDir::new(&current_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("md")) {
            let path = entry.path().strip_prefix(config.docs_dir()).unwrap();

            docs.push(Document::load(entry.path(), path, config.base_path()));
        } else {
            let path = entry.into_path();

            if path.as_path() == current_dir {
                continue;
            }

            if let Some(dir) = walk_dir(path, config) {
                dirs.push(dir);
            }
        }
    }

    if docs.is_empty() {
        None
    } else {
        Some(Directory {
            path: current_dir.to_path_buf(),
            docs,
            dirs,
        })
    }
}

fn generate_missing_indices(dir: &mut Directory, config: &Config) {
    if dir
        .docs
        .iter()
        .find(|d| d.original_file_name() == Some(OsStr::new("README.md")))
        .is_none()
    {
        let new_index = generate_missing_index(dir, config);
        dir.docs.push(new_index);
    }

    for mut child in &mut dir.dirs {
        generate_missing_indices(&mut child, config);
    }
}

fn generate_missing_index(dir: &mut Directory, config: &Config) -> Document {
    let content = dir
        .docs
        .iter()
        .map(|d| format!("* [{}]({})", d.title(), d.uri_path()))
        .collect::<Vec<_>>()
        .join("\n");

    let mut frontmatter = BTreeMap::new();
    frontmatter.insert(
        "title".to_string(),
        format!("{}", dir.path().file_name().unwrap().to_string_lossy()),
    );

    let tmp = dir.path().join("README.md");
    let path = tmp.strip_prefix(config.docs_dir()).unwrap();

    Document::new(
        path,
        format!(
            "# Index of {}\n \
                \n \
                This page was generated automatically by Doctave, because the directory \
                `{}` did not contain an index `README.md` file. You can customize this page by \
                creating one yourself.\
                \n\
                ## Pages\n\
                \n\
                {}",
            dir.path().file_name().unwrap().to_string_lossy(),
            dir.path()
                .strip_prefix(config.project_root())
                .unwrap_or_else(|_| dir.path())
                .display(),
            content
        ),
        frontmatter,
        config.base_path(),
    )
}
