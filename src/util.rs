use crate::Event;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn log_event(e: &Event) {
    log::debug!("received event 🎉: {:#?}", e);
}

pub fn log_dead() {
    log::debug!("sender disconnected! the watcher is dead 💀");
}

pub fn handler_for_event<'a, H>(
    e: &Event,
    handlers: &'a mut HashMap<PathBuf, H>,
) -> Option<&'a mut H> {
    fn path_from_event(e: &Event) -> Option<&Path> {
        match e {
            Event::NoticeWrite(p)
            | Event::NoticeRemove(p)
            | Event::Create(p)
            | Event::Write(p)
            | Event::Chmod(p)
            | Event::Remove(p)
            | Event::Rename(p, _) => Some(p.as_path()),
            _ => None,
        }
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
