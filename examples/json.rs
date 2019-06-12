use hotwatch::{Event, Hotwatch};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Debug, serde::Deserialize)]
struct CuteGirl {
    name: String,
    hobby: String,
    favorite_color: String,
}

fn main() -> Result<(), failure::Error> {
    let mut watcher = Hotwatch::new()?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/data.json");
    let changed = AtomicBool::new(true).into();
    {
        let changed = Arc::clone(&changed);
        watcher.watch(&path, move |event| match event {
            Event::Write(_) => changed.store(true, Ordering::Release),
            _ => (),
        })?;
    }
    loop {
        if changed.compare_and_swap(true, false, Ordering::AcqRel) {
            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);
            match serde_json::from_reader::<_, Vec<CuteGirl>>(&mut reader) {
                Ok(cute_girls) => println!("{:#?}", cute_girls),
                Err(err) => println!("failed to deserialize json: {:#?}", err),
            }
        }
    }
}
