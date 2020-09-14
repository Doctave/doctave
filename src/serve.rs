use std::io;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::time::Duration;

use futures_util::future;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use hyper_staticfile::Static;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use tokio::runtime::Runtime;

use crate::site::Site;

pub struct ServeCommand {
    project_root: PathBuf,
    site: Site,
    threads: Vec<JoinHandle<()>>,
}

impl ServeCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let mut cmd = ServeCommand {
            project_root: root.clone(),
            site: Site::in_dir(root.join("site")),
            threads: Vec::new(),
        };

        cmd.site.build_from(&cmd.project_root)?;

        cmd.threads.push(cmd.watch_files(vec![
            cmd.project_root.join("README.md"),
            cmd.project_root.join("docs"),
        ]));
        cmd.run_server()?;

        Ok(())
    }

    fn watch_files(&self, paths: Vec<PathBuf>) -> JoinHandle<()> {
        std::thread::spawn(|| {
            let (tx, rx) = channel();

            // Create a watcher object, delivering debounced events.
            // The notification back-end is selected based on the platform.
            let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

            for path in paths {
                watcher.watch(path, RecursiveMode::Recursive).unwrap();
            }

            fn notify(path: PathBuf, msg: &str) {
                println!("  {} {}.", path.display(), msg);
            }

            loop {
                match rx.recv() {
                    Ok(event) => match event {
                        DebouncedEvent::NoticeWrite(_) => {}
                        DebouncedEvent::NoticeRemove(_) => {}
                        DebouncedEvent::Create(p) => notify(p, "created"),
                        DebouncedEvent::Write(p) => notify(p, "updated"),
                        DebouncedEvent::Chmod(p) => notify(p, "updated"),
                        DebouncedEvent::Remove(p) => notify(p, "deleted"),
                        DebouncedEvent::Rename(p, new) => {
                            notify(p, &format!("renamed to {}", new.display()))
                        }
                        _ => {}
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        })
    }

    fn run_server(&self) -> io::Result<()> {
        // Create the runtime
        let mut rt = Runtime::new()?;

        println!("Starting server...");

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
    static_.clone().serve(req).await
}
