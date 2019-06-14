fn require_send<T: Send>() {}
fn require_sync<T: Sync>() {}

#[test]
fn hotwatch_send() {
    require_send::<hotwatch::Hotwatch>();
}

#[test]
fn hotwatch_sync() {
    require_sync::<hotwatch::Hotwatch>();
}
