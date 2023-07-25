//! Blocking file watching

use crate::{util, Error, Event, RECURSIVE_MODE};
use notify::Watcher as _;
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flow {
    /// Continue watching and blocking the thread.
    Continue,
    /// Stop watching, returning control of the thread.
    Exit,
}

impl Default for Flow {
    fn default() -> Self {
        Self::Continue
    }
}

/// A blocking hotwatch instance.
///
/// No watching will actually happen until you call [`Hotwatch::run`], which blocks
/// the thread until a handler returns [`Flow::Exit`]. This is useful if you just
/// want to wait on some criteria, rather than if you're building some long-running
/// sexy hot reload service.
///
/// Dropping this will unwatch everything.
pub struct Hotwatch {
    debouncer: Debouncer<notify::RecommendedWatcher, FileIdMap>,
    handlers: HashMap<PathBuf, Box<dyn FnMut(Event) -> Flow>>,
    rx: Receiver<DebounceEventResult>,
}

impl std::fmt::Debug for Hotwatch {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Hotwatch").finish()
    }
}

impl Hotwatch {
    /// Creates a new blocking hotwatch instance.
    ///
    /// # Errors
    ///
    /// This will fail if the underlying [notify](https://docs.rs/notify/4.0/notify/)
    /// instance fails to initialize.
    ///
    /// # Examples
    ///
    /// ```
    /// use hotwatch::blocking::Hotwatch;
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
        let debouncer = new_debouncer(delay, None, tx).map_err(Error::Notify)?;
        Ok(Self {
            debouncer,
            handlers: Default::default(),
            rx,
        })
    }

    /// Watch a path and register a handler to it.
    ///
    /// Handlers won't actually be run until you call [`Hotwatch::run`].
    ///
    /// When watching a directory, that handler will receive all events for all directory
    /// contents, even recursing through subdirectories.
    ///
    /// Only the most specific applicable handler will be called. In other words, if you're
    /// watching "dir" and "dir/file1", then only the latter handler will fire for changes to
    /// `file1`.
    ///
    /// # Errors
    ///
    /// Watching will fail if the path can't be read, returning [`Error::Io`].
    ///
    /// # Examples
    ///
    /// ```
    /// use hotwatch::{
    ///     blocking::{Flow, Hotwatch},
    ///     notify::event::ModifyKind,
    ///     Event, EventKind,
    /// };
    ///
    /// let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    /// // Note that this won't actually do anything until you call `hotwatch.run()`!
    /// hotwatch.watch("README.md", |event: Event| {
    ///     if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
    ///         println!("{:?} changed!", event.paths[0]);
    ///         Flow::Exit
    ///     } else {
    ///         Flow::Continue
    ///     }
    /// }).expect("failed to watch file!");
    /// ```
    pub fn watch<P, F>(&mut self, path: P, handler: F) -> Result<(), Error>
    where
        P: AsRef<Path>,
        F: 'static + FnMut(Event) -> Flow,
    {
        let absolute_path = path.as_ref().canonicalize()?;
        self.debouncer
            .watcher()
            .watch(&absolute_path, RECURSIVE_MODE)?;
        self.debouncer
            .cache()
            .add_root(&absolute_path, RECURSIVE_MODE);
        self.handlers.insert(absolute_path, Box::new(handler));
        Ok(())
    }

    /// Stop watching a path.
    ///
    /// # Errors
    ///
    /// This will fail if the path wasn't being watched, or if the path
    /// couldn't be unwatched for some platform-specific internal reason.
    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let absolute_path = path.as_ref().canonicalize()?;
        self.debouncer.watcher().unwatch(&absolute_path)?;
        self.debouncer.cache().remove_root(&absolute_path);
        self.handlers.remove(&absolute_path);
        Ok(())
    }

    /// Run handlers in an endless loop, blocking the thread.
    ///
    /// The loop will only exit if a handler returns [`Flow::Exit`].
    pub fn run(&mut self) {
        'watch: loop {
            match self.rx.recv() {
                Ok(result) => match result {
                    Ok(events) => {
                        for event in events {
                            util::log_event(&event);
                            if let Some(handler) =
                                util::handler_for_event(&event, &mut self.handlers)
                            {
                                if let Flow::Exit = handler(event) {
                                    break 'watch;
                                }
                            }
                        }
                    }
                    Err(errs) => {
                        for err in errs {
                            util::log_error(&err);
                        }
                    }
                },
                Err(_) => {
                    util::log_dead();
                    break;
                }
            }
        }
    }
}
