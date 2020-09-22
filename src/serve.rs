use std::io;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

use colored::*;
use crossbeam_channel::{bounded, Sender};

use crate::config::Config;
use crate::livereload_server::LivereloadServer;
use crate::preview_server::PreviewServer;
use crate::site::Site;
use crate::watcher::Watcher;

pub struct ServeCommand {
    config: Config,
    site: Site,
    listeners: Arc<RwLock<Vec<Sender<()>>>>,
}

impl ServeCommand {
    pub fn run(config: Config) -> io::Result<()> {
        let site = Site::in_dir(config.out_dir());

        let cmd = ServeCommand {
            config,
            site,
            listeners: Arc::new(RwLock::new(Vec::with_capacity(8))),
        };

        println!("{}", "Doctave CLI | Serve".blue().bold());
        println!("🚀 Starting development server...\n");

        // Do initial build ---------------------------

        let start = Instant::now();
        cmd.site.build_from(&cmd.config.project_root()).unwrap();
        let duration = start.elapsed();

        // Watcher ------------------------------------

        let (watch_snd, watch_rcv) = bounded(128);
        let watcher = Watcher::new(
            vec![
                cmd.config.project_root().join("README.md"),
                cmd.config.project_root().join("docs"),
            ],
            watch_snd,
        );
        thread::Builder::new()
            .name("watcher".into())
            .spawn(move || watcher.run())
            .unwrap();

        // Live Reload --------------------------------

        let (reload_send, reload_rcv) = bounded(128);
        let livereload_server = LivereloadServer::new(reload_rcv);
        thread::Builder::new()
            .name("livereload".into())
            .spawn(move || livereload_server.run())
            .unwrap();

        // Preview Server -----------------------------

        let http_server = PreviewServer::new("0.0.0.0:4001", &cmd.site.out_dir());
        thread::Builder::new()
            .name("http-server".into())
            .spawn(move || http_server.run())
            .unwrap();

        // Listen for updates on from the watcher, rebuild the site,
        // and inform the websocket listeners.

        for (path, msg) in watch_rcv {
            println!("    File {} {}.", path.display().to_string().bold(), msg);

            let start = Instant::now();
            cmd.site.build_from(cmd.config.project_root()).unwrap();
            let duration = start.elapsed();

            println!("    Site rebuilt in {}\n", format!("{:?}", duration).bold());

            reload_send.send(()).unwrap();
        }

        Ok(())
    }
}
