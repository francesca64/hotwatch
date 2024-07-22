use hotwatch::{notify::event::ModifyKind, EventKind, Hotwatch};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct CuteGirl {
    name: String,
    hobby: String,
    favorite_color: String,
}

fn main() -> Result<(), failure::Error> {
    let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_secs_f32(0.5))?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/data.json");
    let changed = AtomicBool::new(true).into();
    {
        let changed = Arc::clone(&changed);
        watcher.watch(&path, move |event| {
            if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
                changed.store(true, Ordering::Release);
            }
        })?;
    }
    loop {
        if changed
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);
            match serde_json::from_reader::<_, Vec<CuteGirl>>(&mut reader) {
                Ok(cute_girls) => println!("{cute_girls:#?}"),
                Err(err) => println!("failed to deserialize json: {err}"),
            }
        }
    }
}
