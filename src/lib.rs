#![feature(box_syntax)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate notify;
extern crate parking_lot;

mod event;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use parking_lot::Mutex;
use notify::{RecommendedWatcher, Watcher};
use event::event_from_notify;

pub type Event = event::Event;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Notify(notify::Error),
    Receive(std::sync::mpsc::RecvError)
}

type HotwatchResult<T> = Result<T, Error>;

type Handler = Box<Fn(Event) + Send>;
type HandlerMapMutex = Arc<Mutex<HashMap<String, Handler>>>;
type ErrorHandler = Box<Fn(Error) + Send>;
type ErrorHandlerMutex = Arc<Mutex<ErrorHandler>>;

pub struct Hotwatch {
    watcher: RecommendedWatcher,
    handler_map_mutex: HandlerMapMutex,
    error_handler_mutex: ErrorHandlerMutex
}

impl Hotwatch {
    pub fn new() -> HotwatchResult<Self> {
        let (tx, rx) = channel();
        let handler_map_mutex = Arc::new(Mutex::new(HashMap::new()));
        let error_handler: ErrorHandler = box |e: Error| { println!("{:?}", e); };
        let error_handler_mutex = Arc::new(Mutex::new(error_handler));
        Hotwatch::run(handler_map_mutex.clone(), error_handler_mutex.clone(), rx);
        Watcher::new(tx)
            .map_err(Error::Notify)
            .map(|watcher| Hotwatch {
                watcher: watcher,
                handler_map_mutex: handler_map_mutex,
                error_handler_mutex: error_handler_mutex
            })
    }

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

    pub fn error_handler<F>(&mut self, handler: F)
    where F: 'static + Fn(Error) + Send {
        let mut error_handler = self.error_handler_mutex.lock();
        *error_handler = box handler;
    }

    fn run(
        handler_map_mutex: HandlerMapMutex,
        error_handler_mutex: ErrorHandlerMutex,
        rx: Receiver<notify::Event>
    ) {
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
                        let error_handler = error_handler_mutex.lock();
                        (*error_handler)(Error::Receive(e));
                    }
                }
            }
        });
    }
}
