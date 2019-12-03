use crate::Event;
use std::{collections::HashMap, path::PathBuf};

pub fn log_event(e: &Event) {
    log::debug!("received event 🎉: {:#?}", e);
}

pub fn log_dead() {
    log::debug!("sender disconnected! the watcher is dead 💀");
}

pub fn handler_for_event<'a, H>(e: &Event, handlers: &'a HashMap<PathBuf, H>) -> Option<&'a H> {
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

    fn find_handler<'a, H>(mut path: PathBuf, handlers: &'a HashMap<PathBuf, H>) -> Option<&'a H> {
        let mut handler = None;
        let mut poppable = true;
        while handler.is_none() && poppable {
            log::debug!("matching against {:?}", path);
            handler = handlers.get(&path);
            poppable = path.pop();
        }
        handler
    }

    path_from_event(e).and_then(|path| find_handler(path, handlers))
}
