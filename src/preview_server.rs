use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ascii::AsciiString;
use bunt::termcolor::{ColorChoice, StandardStream};
use tiny_http::{Request, Response, Server};

use crate::site::{Site, SiteBackend};

pub struct PreviewServer<B: SiteBackend> {
    color: bool,
    base_path: String,
    addr: SocketAddr,
    site: Arc<Site<B>>,
}

impl<B: SiteBackend> PreviewServer<B> {
    pub fn new(addr: &str, site: Arc<Site<B>>, color: bool, base_path: String) -> Self {
        PreviewServer {
            addr: addr.parse().expect("invalid address for preview server"),
            site,
            color,
            base_path,
        }
    }

    pub fn run(self) {
        let server = Server::http(&self.addr).unwrap();
        let mut pool = scoped_threadpool::Pool::new(16);

        {
            let mut stdout = if self.color {
                StandardStream::stdout(ColorChoice::Auto)
            } else {
                StandardStream::stdout(ColorChoice::Never)
            };

            bunt::writeln!(
                stdout,
                "Server running on {$bold}http://{}{}{/$}\n",
                self.addr,
                self.base_path
            )
            .unwrap();
        }

        for request in server.incoming_requests() {
            pool.scoped(|scope| {
                scope.execute(|| {
                    handle_request(request, &self.site, &self.base_path);
                });
            })
        }
    }
}

fn handle_request<B: SiteBackend>(request: Request, site: &Site<B>, base_path: &str) {
    let result = {
        let uri = request.url().parse::<http::Uri>().unwrap();

        let path = PathBuf::from(uri.path());

        match resolve_file(&path, &site, base_path)
            .map(|p| (read_file(site, &p), content_type_for(p.extension())))
        {
            Some((data, None)) => request.respond(Response::from_data(data).with_status_code(200)),
            Some((data, Some(content_type))) => {
                request.respond(Response::from_data(data).with_status_code(200).with_header(
                    tiny_http::Header {
                        field: "Content-Type".parse().unwrap(),
                        value: AsciiString::from_ascii(content_type).unwrap(),
                    },
                ))
            }
            None => request.respond(Response::new_empty(tiny_http::StatusCode(404))),
        }
    };

    match result {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
        Err(e) => eprintln!("    HTTP server threw error: {}", e),
    }
}

/// Uses some basic logic for resolving a path into the correct file.
/// This means resolving to an index.html from the root of the directory,
/// trying with .html extensions with needed, etc.
pub fn resolve_file<B: SiteBackend>(
    path: &Path,
    site: &Site<B>,
    base_path: &str,
) -> Option<PathBuf> {
    if path.to_str().map(|s| s.contains("..")).unwrap_or(false) {
        return None;
    }

    let mut path = path;

    if path.to_str().map(|s| s.contains("#")).unwrap_or(false) {
        let prefix = path.to_str().unwrap().split("#").next().unwrap();

        path = Path::new(prefix);
    }

    if path.starts_with(base_path) {
        path = path.strip_prefix(base_path).unwrap();
    } else {
        return None;
    }

    let mut path = path.strip_prefix("/").unwrap_or(path).to_owned();

    if site.backend.has_file(&path) {
        Some(path)
    } else if site.backend.has_file(&path.join("index.html")) {
        let p = path.join("index.html");

        Some(p)
    } else {
        // Try with a .html extension
        path.set_extension("html");

        if site.backend.has_file(&path) {
            Some(path)
        } else {
            None
        }
    }
}

fn read_file<B: SiteBackend>(site: &Site<B>, path: &Path) -> Vec<u8> {
    let content = site
        .backend
        .read_path(path)
        .expect("Found a file to serve but could not open it");

    content
}

fn content_type_for(extension: Option<&OsStr>) -> Option<&'static str> {
    match extension {
        Some(s) => match s.to_str() {
            Some("txt") => Some("text/plain; charset=utf8"),
            Some("html") => Some("text/html; charset=utf8"),
            Some("htm") => Some("text/html; charset=utf8"),
            Some("css") => Some("text/css"),
            Some("js") => Some("text/javascript"),
            Some("pdf") => Some("application/pdf"),
            Some("zip") => Some("application/zip"),
            Some("jpg") => Some("image/jpeg"),
            Some("jpeg") => Some("image/jpeg"),
            Some("png") => Some("image/png"),
            Some("svg") => Some("image/svg+xml"),
            None => None,
            _ => None,
        },
        None => None,
    }
}
