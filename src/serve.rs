use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use futures_util::future;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use hyper_staticfile::Static;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use tokio::runtime::Runtime;
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
                watcher.watch(path, RecursiveMode::Recursive).unwrap();
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
        // Create the runtime
        let mut rt = Runtime::new()?;

        // Spawn the root task
        rt.block_on(async {
            let static_ = Static::new(self.site.out_dir());

            let make_service = make_service_fn(|_| {
                let static_ = static_.clone();
                future::ok::<_, hyper::Error>(service_fn(move |req| {
                    handle_request(req, static_.clone())
                }))
            });

            let addr = ([127, 0, 0, 1], 4001).into();
            let server = hyper::Server::bind(&addr).serve(make_service);
            eprintln!("Doc server running on http://{}/", addr);
            server.await.expect("Server failed");
        });

        Ok(())
    }
}

async fn handle_request<B>(req: Request<B>, static_: Static) -> Result<Response<Body>, io::Error> {
    if req.uri().path() == "/livereload.js" {
        Ok(Response::new(Body::from(crate::LIVERELOAD_JS)))
    } else {
        static_.clone().serve(req).await
    }
}

fn handle_websocket(stream: std::net::TcpStream, broadcast_listener: Receiver<()>) {
    let result = || -> io::Result<()> {
        let mut websocket = tungstenite::accept(stream).unwrap();

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
    site.build_from(project_root).unwrap();

    let list = listeners.read().unwrap();

    for l in &*list {
        l.send(()).unwrap();
    }

    println!("  {} {}.", path.display(), msg);
}
