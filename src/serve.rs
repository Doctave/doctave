use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use ascii::AsciiString;
use colored::*;
use crossbeam_channel::{bounded, Receiver, Sender};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use tiny_http::{Request, Response, Server};
use tungstenite::protocol::WebSocket;

use crate::site::Site;

pub struct ServeCommand {
    project_root: PathBuf,
    site: Site,
    threads: Vec<JoinHandle<()>>,
    listeners: Arc<RwLock<Vec<Sender<()>>>>,
}

impl ServeCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let mut cmd = ServeCommand {
            project_root: root.clone(),
            site: Site::in_dir(root.join("site")),
            threads: Vec::new(),
            listeners: Arc::new(RwLock::new(Vec::with_capacity(8))),
        };

        println!("{}", "Doctave CLI | Serve".blue().bold());
        println!("🚀 Starting development server...\n");

        cmd.threads.push(cmd.builder_thread(
            Site::in_dir(cmd.project_root.join("site")),
            cmd.project_root.clone(),
            vec![
                cmd.project_root.join("README.md"),
                cmd.project_root.join("docs"),
            ],
        ));
        cmd.threads.push(cmd.livereload_server());
        cmd.run_server()?;

        Ok(())
    }

    fn builder_thread(&self, site: Site, root: PathBuf, paths: Vec<PathBuf>) -> JoinHandle<()> {
        let listeners = self.listeners.clone();

        std::thread::spawn(move || {
            let (tx, rx) = channel();
            let site = site;
            let root = root;

            site.build_from(&root).unwrap();

            // Create a watcher object, delivering debounced events.
            // The notification back-end is selected based on the platform.
            let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

            for path in paths {
                if path.exists() {
                    watcher.watch(path, RecursiveMode::Recursive).unwrap();
                }
            }

            loop {
                match rx.recv() {
                    Ok(event) => match event {
                        DebouncedEvent::NoticeWrite(_) => {}
                        DebouncedEvent::NoticeRemove(_) => {}
                        DebouncedEvent::Create(p) => {
                            rebuild(&site, &root, p, "created", listeners.clone())
                        }
                        DebouncedEvent::Write(p) => {
                            rebuild(&site, &root, p, "updated", listeners.clone())
                        }
                        DebouncedEvent::Chmod(p) => {
                            rebuild(&site, &root, p, "updated", listeners.clone())
                        }
                        DebouncedEvent::Remove(p) => {
                            rebuild(&site, &root, p, "deleted", listeners.clone())
                        }
                        DebouncedEvent::Rename(p, new) => rebuild(
                            &site,
                            &root,
                            p,
                            &format!("renamed to {}", new.display()),
                            listeners.clone(),
                        ),
                        _ => {}
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        })
    }

    fn livereload_server(&self) -> JoinHandle<()> {
        let listeners = self.listeners.clone();

        std::thread::spawn(move || {
            let server = std::net::TcpListener::bind("127.0.0.1:35729").unwrap();

            for stream in server.incoming().filter_map(Result::ok) {
                let (sender, receiver) = bounded(128);

                {
                    let mut list = listeners.write().unwrap();
                    list.push(sender);
                }

                std::thread::spawn(move || {
                    handle_websocket(stream, receiver);
                });
            }
        })
    }

    fn run_server(&self) -> io::Result<()> {
        let addr = "0.0.0.0:4001";
        let server = Server::http(&addr).unwrap();

        println!("Server running on {}\n", format!("http://{}/", addr).bold());

        for request in server.incoming_requests() {
            self.handle_request(request);
        }

        Ok(())
    }

    fn handle_request(&self, request: Request) {
        let result = {
            let uri = request.url().parse::<http::Uri>().unwrap();

            match self.resolve_file(&Path::new(uri.path())) {
                Some((f, None)) => request
                    .respond(Response::from_file(File::open(f).unwrap()).with_status_code(200)),
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

    fn resolve_file(&self, path: &Path) -> Option<(PathBuf, Option<&str>)> {
        if path.to_str().map(|s| s.contains("..")).unwrap_or(false) {
            return None;
        }

        let mut components = path.components();
        components.next();
        let path = self.site.out_dir().join(components.as_path());

        if path.is_file() && path.exists() {
            Some((path.to_path_buf(), self.content_type_for(path.extension())))
        } else if path.is_dir() && path.join("index.html").exists() {
            let p = path.join("index.html");
            let extension = p.extension();

            Some((p.clone(), self.content_type_for(extension)))
        } else {
            None
        }
    }

    fn content_type_for(&self, extension: Option<&OsStr>) -> Option<&'static str> {
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
}

fn handle_websocket(stream: std::net::TcpStream, broadcast_listener: Receiver<()>) {
    let result = || -> io::Result<()> {
        let mut websocket =
            tungstenite::accept(stream).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        livereload_handshake(&mut websocket)?;

        for _ in broadcast_listener.iter() {
            websocket
                .write_message(
                    r#"
                    {
                        "command": "reload",
                        "path": "",
                        "liveCSS": true
                    }
                    "#
                    .into(),
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        Ok(())
    };

    match (result)() {
        Ok(_) => println!("Livereload client disconnected."),
        Err(e) => println!("Livereload client disconnected due to an error: {}.", e),
    };
}

fn livereload_handshake(websocket: &mut WebSocket<std::net::TcpStream>) -> io::Result<()> {
    let msg = websocket.read_message().unwrap();

    if msg.is_text() {
        let parsed: serde_json::Value = serde_json::from_str(msg.to_text().unwrap())?;

        if parsed["command"] != "hello" {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid handshake"));
        }

        let response = r#"
        {
            "command": "hello",
            "protocols": ["http://livereload.com/protocols/official-7"],
            "serverName": "doctave"
        }
        "#;

        websocket
            .write_message(response.into())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Invalid handshake"))
    }
}

fn rebuild(
    site: &Site,
    project_root: &Path,
    path: PathBuf,
    msg: &str,
    listeners: Arc<RwLock<Vec<Sender<()>>>>,
) {
    use std::time::Instant;

    println!("    File {} {}.", path.display().to_string().bold(), msg);

    let start = Instant::now();
    site.build_from(project_root).unwrap();
    let duration = start.elapsed();

    println!("    Site rebuilt in {}\n", format!("{:?}", duration).bold());

    let list = listeners.read().unwrap();

    for l in &*list {
        l.send(()).unwrap();
    }
}
