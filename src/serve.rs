use std::io;
use std::path::PathBuf;

use futures_util::future;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use hyper_staticfile::Static;
use tokio::runtime::Runtime;

use crate::site::Site;

pub struct ServeCommand {
    project_root: PathBuf,
    site: Site,
}

impl ServeCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let cmd = ServeCommand {
            project_root: root.clone(),
            site: Site::in_dir(root.join("site")),
        };

        cmd.site.build_from(&cmd.project_root)?;

        cmd.run_server()?;

        Ok(())
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

            let addr = ([127, 0, 0, 1], 3000).into();
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
