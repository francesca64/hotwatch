//! `hotwatch` is a Rust library for comfortably watching and handling file changes.
//! It's a thin convenience wrapper over [`notify`](https://github.com/passcod/notify),
//! allowing you to easily set callbacks for each path you want to watch.
//!
//! Watching is done on a separate thread so you don't have to worry about blocking.
//! All handlers are run on that thread as well, so keep that in mind when attempting to access
//! outside data from within a handler.
//!
//! Only the latest stable version of Rust is supported.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
};

use notify::Watcher as _;
pub use notify::{self, DebouncedEvent as Event};

fn path_from_event(e: &Event) -> Option<PathBuf> {
    match e {
        Event::NoticeWrite(p)
        | Event::NoticeRemove(p)
        | Event::Create(p)
        | Event::Write(p)
        | Event::Chmod(p)
        | Event::Remove(p)
        | Event::Rename(p, _) => Some(p.clone()),
        _ => None,
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Notify(notify::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(error) => error.fmt(fmt),
            Error::Notify(error) => error.fmt(fmt),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => error.source(),
            Error::Notify(error) => error.source(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        if let notify::Error::Io(err) = err {
            err.into()
        } else {
            Error::Notify(err)
        }
    }
}

type Handler = Box<Fn(Event) + Send>;
type HandlerMap = HashMap<PathBuf, Handler>;

pub struct Hotwatch {
    watcher: notify::RecommendedWatcher,
    handlers: Arc<Mutex<HandlerMap>>,
}

impl std::fmt::Debug for Hotwatch {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Hotwatch").finish()
    }
}

impl Hotwatch {
    /// Creates a new hotwatch instance.
    ///
    /// # Errors
    ///
    /// This will fail if the underlying [notify](https://docs.rs/notify/4.0/notify/)
    /// instance fails to initialize.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotwatch::Hotwatch;
    ///
    /// let hotwatch = Hotwatch::new().expect("hotwatch failed to initialize");
    /// ```
    pub fn new() -> Result<Self, Error> {
        Self::new_with_custom_delay(std::time::Duration::from_secs(2))
    }

    /// Using [`Hotwatch::new`] will give you a default delay of 2 seconds.
    /// This method allows you to specify your own value.
    ///
    /// # Notes
    ///
    /// A delay of over 30 seconds will prevent repetitions of previous events on macOS.
    pub fn new_with_custom_delay(delay: std::time::Duration) -> Result<Self, Error> {
        let (tx, rx) = channel();
        let handlers = Arc::<Mutex<_>>::default();
        Hotwatch::run(Arc::clone(&handlers), rx);
        notify::Watcher::new(tx, delay)
            .map_err(Error::Notify)
            .map(|watcher| Hotwatch { watcher, handlers })
    }

    /// Watch a path and register a handler to it.
    ///
    /// When watching a directory, that handler will receive all events for all directory
    /// contents, even recursing through subdirectories.
    ///
    /// Only the most specific applicable handler will be called. In other words, if you're
    /// watching "dir" and "dir/file1", then only the latter handler will fire for changes to
    /// `file1`.
    ///
    /// Note that handlers will be run in hotwatch's watch thread, so you'll have to use `move`
    /// if the closure captures anything.
    ///
    /// # Errors
    ///
    /// Watching will fail if the path can't be read, returning [`Error::Io`].
    ///
    /// # Examples
    ///
    /// ```
    /// use hotwatch::{Hotwatch, Event};
    ///
    /// let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    /// hotwatch.watch("README.md", |event: Event| {
    ///     if let Event::Write(path) = event {
    ///         println!("{:?} changed!", path);
    ///     }
    /// }).expect("failed to watch file!");
    /// ```
    pub fn watch<P, F>(&mut self, path: P, handler: F) -> Result<(), Error>
    where
        P: AsRef<Path>,
        F: 'static + Fn(Event) + Send,
    {
        let absolute_path = path.as_ref().canonicalize()?;
        let mut handlers = self.handlers.lock().expect("handler mutex poisoned!");
        self.watcher
            .watch(&absolute_path, notify::RecursiveMode::Recursive)?;
        (*handlers).insert(absolute_path, Box::new(handler));
        log::debug!("active handlers: {:#?}", handlers.keys());
        Ok(())
    }

    fn run(handlers: Arc<Mutex<HandlerMap>>, rx: Receiver<Event>) {
        std::thread::spawn(move || loop {
            match rx.recv() {
                Ok(event) => {
                    log::debug!("received event 🎉: {:#?}", event);
                    let handlers = handlers.lock().expect("handler mutex poisoned!");
                    if let Some(mut path) = path_from_event(&event) {
                        let mut handler = None;
                        let mut poppable = true;
                        while handler.is_none() && poppable {
                            log::debug!("matching against {:?}", path);
                            handler = (*handlers).get(&path);
                            poppable = path.pop();
                        }
                        if let Some(handler) = handler {
                            handler(event);
                        }
                    }
                }
                Err(_) => log::error!("sender disconnected! the watcher is dead 💀"),
            }
        });
    }
}
