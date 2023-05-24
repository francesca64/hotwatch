use hotwatch::{
    blocking::{Flow, Hotwatch},
    EventKind,
};
use std::path::Path;

fn main() -> Result<(), failure::Error> {
    let mut watcher = Hotwatch::new()?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/data.json");
    watcher.watch(&path, move |event| {
        if let EventKind::Modify(_) = event.kind {
            Flow::Exit
        } else {
            Flow::Continue
        }
    })?;
    println!("Edit data.json, and thou shalt be rewarded...");
    watcher.run();
    println!("🌭 🍔 🍟");
    Ok(())
}
