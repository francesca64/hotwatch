//! hotwatch is a Rust library for conveniently watching and handling file changes.
//!
//! Watching is done on a separate thread to avoid blocking your enjoyment of life.
//! All handlers are run on that thread as well, so keep that in mind when attempting to access
//! outside data from within a handler.
//!
//! Nightly Rust is required, as hotwatch uses the box keyword internally.

#![feature(box_syntax)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate notify;
extern crate parking_lot;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use parking_lot::Mutex;
use notify::{RecommendedWatcher, Watcher};

#[derive(Debug)]
pub struct Event {
    pub path: String,
    op_flags: u32
}

impl Event {
    fn check(&self, op: notify::op::Op) -> bool {
        let flag = op.bits();
        self.op_flags & flag == flag
    }

    pub fn chmoded(&self) -> bool {
        self.check(notify::op::CHMOD)
    }

    pub fn created(&self) -> bool {
        self.check(notify::op::CREATE)
    }

    pub fn changed(&self) -> bool {
        self.check(notify::op::WRITE)
    }

    pub fn removed(&self) -> bool {
        self.check(notify::op::REMOVE)
    }

    pub fn renamed(&self) -> bool {
        self.check(notify::op::RENAME)
    }

    pub fn ignored(&self) -> bool {
        self.check(notify::op::IGNORED)
    }
}

fn event_from_notify(e: notify::Event) -> Option<Event> {
    if let notify::Event { path: Some(path), op: Ok(op) } = e {
        if cfg!(debug_assertions) {
            println!("{:?} {:?}", op, path);
        }
        path.to_str()
            .map(|s| s.to_string())
            .map(|s| Event {
                path: s,
                op_flags: op.bits()
            })
    } else {
        None
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
    /// This function can fail if the underlying [notify](https://github.com/passcod/rsnotify)
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
        Watcher::new(tx)
            .map_err(Error::Notify)
            .map(|watcher| Hotwatch {
                watcher: watcher,
                handler_map_mutex: handler_map_mutex,
            })
    }

    /// Watch a path and register a handler to it.
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
    /// use hotwatch::Hotwatch;
    ///
    /// let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
    /// hotwatch.watch("README.md", |e: hotwatch::Event| {
    ///   if e.changed() {
    ///     println!("{} changed!", e.path);
    ///   }
    /// }).expect("Failed to watch file!");
    /// ```
    pub fn watch<F>(&mut self, path: &str, handler: F) -> HotwatchResult<()>
    where F: 'static + Fn(Event) + Send {
        let mut handlers = self.handler_map_mutex.lock();
        self.watcher.watch(Path::new(path))
            .map_err(|e| match e {
                notify::Error::Io(e) => Error::Io(e),
                _ => Error::Notify(e)
            })
            .map(|_| {
                (*handlers).insert(path.to_string(), box handler);
            })
    }

    fn run(handler_map_mutex: HandlerMapMutex, rx: Receiver<notify::Event>) {
        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(notify_event) => {
                        if let Some(event) = event_from_notify(notify_event) {
                            let handlers = handler_map_mutex.lock();
                            if let Some(handler) = (*handlers).get(&event.path) {
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
