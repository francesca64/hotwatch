//! hotwatch is a Rust library for comfortably watching and handling file changes. 
//! It's a thin convenience wrapper over [notify](https://github.com/passcod/notify), allowing you to easily spawn handlers.
//!
//! Watching is done on a separate thread to avoid blocking your enjoyment of life.
//! All handlers are run on that thread as well, so keep that in mind when attempting to access
//! outside data from within a handler.
//!
//! Nightly Rust is required, as hotwatch uses the box keyword internally.

#![feature(box_syntax)]

extern crate notify;
extern crate parking_lot;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use parking_lot::Mutex;
use notify::{RecommendedWatcher, Watcher, RecursiveMode};

pub use notify::DebouncedEvent as Event;

fn path_from_event(e: &Event) -> Option<PathBuf> {
    match e {
        &Event::NoticeWrite(ref p) |
        &Event::NoticeRemove(ref p) |
        &Event::Create(ref p) |
        &Event::Write(ref p) |
        &Event::Chmod(ref p) |
        &Event::Remove(ref p) |
        &Event::Rename(ref p, _) => Some(p.clone()),
        &_ => None
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Notify(notify::Error)
}

type HotwatchResult<T> = Result<T, Error>;

type Handler = Box<Fn(Event) + Send>;
type HandlerMapMutex = Arc<Mutex<HashMap<String, Handler>>>;

pub struct Hotwatch {
    watcher: RecommendedWatcher,
    handler_map_mutex: HandlerMapMutex,
}

impl Hotwatch {
    /// Creates a new hotwatch instance.
    ///
    /// # Errors
    ///
    /// This function can fail if the underlying [notify](https://docs.rs/notify/4.0.1/notify/)
    /// instance fails to initialize. This will unfortunately expose you to notify's own error
    /// type; hotwatch doesn't perfectly encapsulate this.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotwatch::Hotwatch;
    ///
    /// let hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
    /// ```
    pub fn new() -> HotwatchResult<Self> {
        let (tx, rx) = channel();
        let handler_map_mutex = Arc::new(Mutex::new(HashMap::new()));
        Hotwatch::run(handler_map_mutex.clone(), rx);
        Watcher::new(tx, Duration::from_secs(2))
            .map_err(Error::Notify)
            .map(|watcher| Hotwatch {
                watcher: watcher,
                handler_map_mutex: handler_map_mutex,
            })
    }

    /// Watch a path and register a handler to it.
    ///
    /// When watching a directory, that handler will receive all events for all directory
    /// contents, even recursiving through subdirectories.
    ///
    /// Only the most specific applicable handler will be called. In other words, if you're
    /// watching "dir" and "dir/file1", then only the latter handler will fire for changes to
    /// file1.
    ///
    /// Note that handlers will be run in hotwatch's watch thread, so you'll have to use `move`
    /// if the closure captures anything.
    ///
    /// # Errors
    ///
    /// Watching will fail if the path can't be read, thus returning
    /// a `hotwatch::Error::Io(std::io::Error)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotwatch::{Hotwatch, Event};
    ///
    /// let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
    /// hotwatch.watch("README.md", |e: Event| {
    ///     if let Event::Write(path) = e {
    ///         println!("{:?} changed!", path);
    ///     }
    /// }).expect("Failed to watch file!");
    /// ```
    pub fn watch<F>(&mut self, path: &str, handler: F) -> HotwatchResult<()>
    where F: 'static + Fn(Event) + Send {
        let mut handlers = self.handler_map_mutex.lock();
        self.watcher.watch(Path::new(path), RecursiveMode::Recursive)
            .map_err(|e| match e {
                notify::Error::Io(e) => Error::Io(e),
                _ => Error::Notify(e)
            })
            .map(|_| {
                (*handlers).insert(path.to_string(), box handler);
            })
    }

    fn run(handler_map_mutex: HandlerMapMutex, rx: Receiver<Event>) {
        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(event) => {
                        if cfg!(debug_assertions) {
                            println!("Hotwatch {:?}", event);
                        }
                        let handlers = handler_map_mutex.lock();
                        if let Some(mut path) = path_from_event(&event) {
                            let mut handler = None;
                            let mut poppable = true;
                            while handler.is_none() && poppable {
                                if let Some(str_path) = path.to_str() {
                                    handler = (*handlers).get(str_path);
                                }
                                poppable = path.pop();
                            }
                            if let Some(handler) = handler {
                                handler(event);
                            }
                        }
                    },
                    Err(e) => {
                        if cfg!(debug_assertions) {
                            println!("Receiver error: {:?}", e);
                        }
                    }
                }
            }
        });
    }
}
