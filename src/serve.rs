use std::thread;
use std::time::Instant;
use std::sync::Arc;

use bunt::termcolor::{ColorChoice, StandardStream};
use crossbeam_channel::bounded;

use crate::config::Config;
use crate::livereload_server::LivereloadServer;
use crate::preview_server::PreviewServer;
use crate::site::{Site, InMemorySite};
use crate::watcher::Watcher;
use crate::Result;

pub struct ServeCommand {
}

#[derive(Default)]
pub struct ServeOptions {
    pub port: Option<u32>,
}

impl ServeCommand {
    pub fn run(options: ServeOptions, config: Config) -> Result<()> {
        let mut stdout = if config.color_enabled() {
            StandardStream::stdout(ColorChoice::Auto)
        } else {
            StandardStream::stdout(ColorChoice::Never)
        };
        let site = Arc::new(InMemorySite::new(config.clone()));

        bunt::writeln!(stdout, "{$bold}{$blue}Doctave | Serve{/$}{/$}")?;
        println!("Starting development server...\n");

        // Do initial build ---------------------------

        let start = Instant::now();
        site.build().unwrap();
        let duration = start.elapsed();

        // Watcher ------------------------------------

        let (watch_snd, watch_rcv) = bounded(128);
        let watcher = Watcher::new(vec![config.project_root().join("docs")], watch_snd);
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

        let port = options.port.unwrap_or_else(|| config.port());

        let http_server = PreviewServer::new(
            &format!("0.0.0.0:{}", port),
            site.clone(),
            config.color_enabled(),
        );
        thread::Builder::new()
            .name("http-server".into())
            .spawn(move || http_server.run())
            .unwrap();

        // Listen for updates on from the watcher, rebuild the site,
        // and inform the websocket listeners.

        for (path, msg) in watch_rcv {
            bunt::writeln!(stdout, "    File {$bold}{}{/$} {}.", path.display(), msg)?;

            let start = Instant::now();
            site.build().unwrap();
            let duration = start.elapsed();

            bunt::writeln!(stdout, "    Site rebuilt in {$bold}{:?}{/$}\n", duration)?;

            reload_send.send(()).unwrap();
        }

        Ok(())
    }
}
