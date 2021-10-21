use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ascii::AsciiString;
use bunt::termcolor::{ColorChoice, StandardStream};
use tiny_http::{Request, Response, Server};

use crate::site::{InMemorySite, Site};

pub struct PreviewServer {
    color: bool,
    base_path: Option<PathBuf>,
    addr: SocketAddr,
    site: Arc<InMemorySite>,
}

impl PreviewServer {
    pub fn new(
        addr: &str,
        site: Arc<InMemorySite>,
        color: bool,
        base_path: Option<PathBuf>,
    ) -> Self {
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

            let path = self.base_path.clone().unwrap_or(PathBuf::from("/"));

            bunt::writeln!(
                stdout,
                "Server running on {$bold}http://{}{}{/$}\n",
                self.addr,
                path.display()
            )
            .unwrap();
        }

        for request in server.incoming_requests() {
            pool.scoped(|scope| {
                scope.execute(|| {
                    handle_request(request, &self.site, self.base_path.as_deref());
                });
            })
        }
    }
}

fn handle_request(request: Request, site: &InMemorySite, base_path: Option<&Path>) {
    let result = {
        let uri = request.url().parse::<http::Uri>().unwrap();

        let path = PathBuf::from(uri.path());

        match resolve_file(&path, &site, base_path) {
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

fn resolve_file(
    path: &Path,
    site: &InMemorySite,
    base_path: Option<&Path>,
) -> Option<(Vec<u8>, Option<&'static str>)> {
    if path.to_str().map(|s| s.contains("..")).unwrap_or(false) {
        return None;
    }

    let mut path = path;

    if let Some(base) = base_path {
        if path.starts_with(base) {
            path = path.strip_prefix(base).unwrap();
        } else {
            return None;
        }
    }

    let mut path = path.strip_prefix("/").unwrap_or(path).to_owned();

    if site.has_file(&path) {
        Some((
            site.read_path(&path).unwrap(),
            content_type_for(path.extension()),
        ))
    } else if site.has_file(&path.join("index.html")) {
        let p = path.join("index.html");
        let extension = p.extension();

        Some((site.read_path(&p).unwrap(), content_type_for(extension)))
    } else {
        // Try with a .html extension
        path.set_extension("html");

        if site.has_file(&path) {
            Some((
                site.read_path(&path).unwrap(),
                content_type_for(path.extension()),
            ))
        } else {
            None
        }
    }
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
