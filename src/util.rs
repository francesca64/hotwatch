use crate::Event;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[cfg_attr(not(feature = "logging"), allow(unused_variables))]
pub fn log_event(event: &Event) {
    #[cfg(feature = "logging")]
    log::debug!("received event 🎉: {event:#?}");
}

#[cfg_attr(not(feature = "logging"), allow(unused_variables))]
pub fn log_error(err: &notify::Error) {
    #[cfg(feature = "logging")]
    log::error!("error in event stream: {err}");
}

#[cfg_attr(not(feature = "logging"), allow(unused_variables))]
pub fn log_matching_path(path: &Path) {
    #[cfg(feature = "logging")]
    log::debug!("matching against {:?}", path);
}

pub fn log_dead() {
    #[cfg(feature = "logging")]
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
            log_matching_path(path);
            if handlers.contains_key(path) {
                return handlers.get_mut(path);
            }
            remaining_path = path.parent();
        }
        None
    }

    path_from_event(e).and_then(move |path| find_handler(path, handlers))
}
