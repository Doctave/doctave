use std::ffi::OsStr;
use std::fs::File;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use ascii::AsciiString;
use colored::*;
use tiny_http::{Request, Response, Server};

pub struct PreviewServer {
    addr: SocketAddr,
    out_dir: PathBuf,
}

impl PreviewServer {
    pub fn new<P: Into<PathBuf>>(addr: &str, out_dir: P) -> Self {
        PreviewServer {
            addr: addr.parse().expect("invalid address for preview server"),
            out_dir: out_dir.into(),
        }
    }

    pub fn run(self) {
        let server = Server::http(&self.addr).unwrap();
        let mut pool = scoped_threadpool::Pool::new(16);

        println!(
            "Server running on {}\n",
            format!("http://{}/", self.addr).bold()
        );

        for request in server.incoming_requests() {
            pool.scoped(|scope| {
                scope.execute(|| {
                    handle_request(request, self.out_dir.clone());
                });
            })
        }
    }
}

fn handle_request(request: Request, out_dir: PathBuf) {
    let result = {
        let uri = request.url().parse::<http::Uri>().unwrap();

        match resolve_file(&Path::new(uri.path()), &out_dir) {
            Some((f, None)) => {
                request.respond(Response::from_file(File::open(f).unwrap()).with_status_code(200))
            }
            Some((f, Some(content_type))) => request.respond(
                Response::from_file(File::open(f).unwrap())
                    .with_status_code(200)
                    .with_header(tiny_http::Header {
                        field: "Content-Type".parse().unwrap(),
                        value: AsciiString::from_ascii(content_type).unwrap(),
                    }),
            ),
            None => request.respond(Response::new_empty(tiny_http::StatusCode(404))),
        }
    };

    if let Err(e) = result {
        eprintln!("    HTTP server threw error: {}", e);
    }
}

fn resolve_file(path: &Path, out_dir: &Path) -> Option<(PathBuf, Option<&'static str>)> {
    if path.to_str().map(|s| s.contains("..")).unwrap_or(false) {
        return None;
    }

    let mut components = path.components();
    components.next();
    let path = out_dir.join(components.as_path());

    if path.is_file() && path.exists() {
        Some((path.to_path_buf(), content_type_for(path.extension())))
    } else if path.is_dir() && path.join("index.html").exists() {
        let p = path.join("index.html");
        let extension = p.extension();

        Some((p.clone(), content_type_for(extension)))
    } else {
        None
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
            None => None,
            _ => None,
        },
        None => None,
    }
}
