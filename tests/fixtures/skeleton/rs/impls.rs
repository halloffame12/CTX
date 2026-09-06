pub struct Counter {
    n: u64,
}

impl Default for Counter {
    fn default() -> Self {
        Self { n: 0 }
    }
}

impl Counter {
    pub fn new(n: u64) -> Self {
        Self { n }
    }
}

pub const START: u64 = 100;

static COUNTS: std::sync::Mutex<Vec<u64>> = std::sync::Mutex::new(Vec::new());

pub fn bump() {
    let mut guard = COUNTS.lock().unwrap();
    guard.push(1);
}
