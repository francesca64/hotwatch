use crate::Event;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn log_event(event: &Event) {
    log::debug!("received event 🎉: {event:#?}");
}

pub fn log_error(err: &notify::Error) {
    log::error!("error in event stream: {err}");
}

pub fn log_dead() {
    log::debug!("sender disconnected! the watcher is dead 💀");
}

pub fn handler_for_event<'a, H>(
    e: &Event,
    handlers: &'a mut HashMap<PathBuf, H>,
) -> Option<&'a mut H> {
    fn path_from_event(e: &Event) -> Option<&PathBuf> {
        e.paths.first()
    }

    fn find_handler<'a, H>(
        path: &Path,
        handlers: &'a mut HashMap<PathBuf, H>,
    ) -> Option<&'a mut H> {
        let mut remaining_path = Some(path);
        while let Some(path) = remaining_path {
            log::debug!("matching against {:?}", path);
            if handlers.contains_key(path) {
                return handlers.get_mut(path);
            }
            remaining_path = path.parent();
        }
        None
    }

    path_from_event(e).and_then(move |path| find_handler(path, handlers))
}
