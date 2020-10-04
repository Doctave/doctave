use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use crossbeam_channel::Sender;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher as NotifyWatcher};

pub struct Watcher {
    paths: Vec<PathBuf>,
    channel: Sender<(PathBuf, String)>,
}

impl Watcher {
    pub fn new(paths: Vec<PathBuf>, channel: Sender<(PathBuf, String)>) -> Self {
        Watcher { paths, channel }
    }

    pub fn run(self) {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        for path in &self.paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive).unwrap();
            }
        }

        loop {
            let should_continue = match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::NoticeWrite(_) => true,
                    DebouncedEvent::NoticeRemove(_) => true,
                    DebouncedEvent::Create(p) => self.notify(p, "created"),
                    DebouncedEvent::Write(p) => self.notify(p, "updated"),
                    DebouncedEvent::Chmod(p) => self.notify(p, "updated"),
                    DebouncedEvent::Remove(p) => self.notify(p, "deleted"),
                    DebouncedEvent::Rename(p, new) => {
                        self.notify(p, format!("renamed to {}", new.display()))
                    }
                    _ => true,
                },
                Err(e) => {
                    println!("watch error: {:?}", e);
                    true
                }
            };

            if !should_continue {
                break;
            }
        }
    }

    /// Notifies the listening end (Main thread) that there the paths
    /// being monitored have updated.
    ///
    /// Returns false if the notification could not be send, meaning
    /// the main thread has gone away.
    fn notify<S: Into<String>>(&self, path: PathBuf, msg: S) -> bool {
        self.channel.send((path, msg.into())).is_ok()
    }
}
