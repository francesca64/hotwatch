use notify;

#[derive(Debug)]
pub struct Event {
    pub path: String,
    op_flags: u32
}

impl Event {
    fn check(&self, op: notify::op::Op) -> bool {
        let flag = op.bits();
        self.op_flags & flag == flag
    }

    pub fn chmoded(&self) -> bool {
        self.check(notify::op::CHMOD)
    }

    pub fn created(&self) -> bool {
        self.check(notify::op::CREATE)
    }

    pub fn changed(&self) -> bool {
        self.check(notify::op::WRITE)
    }

    pub fn removed(&self) -> bool {
        self.check(notify::op::REMOVE)
    }

    pub fn rename(&self) -> bool {
        self.check(notify::op::RENAME)
    }

    pub fn ignored(&self) -> bool {
        self.check(notify::op::IGNORED)
    }
}

pub fn event_from_notify(e: notify::Event) -> Option<Event> {
    if let notify::Event { path: Some(path), op: Ok(op) } = e {
        if cfg!(debug_assertions) {
            println!("{:?} {:?}", op, path);
        }
        path.to_str()
            .map(|s| s.to_string())
            .map(|s| Event {
                path: s,
                op_flags: op.bits()
            })
    } else {
        None
    }
}
