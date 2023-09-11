use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub mod time {
    pub fn cur_ts() -> u64 {
        std::time::UNIX_EPOCH.elapsed().unwrap().as_secs()
    }
}

pub struct ShutdownChecker {
    shutdown: Arc<AtomicBool>,
    last_check_ts: u64,
}

impl ShutdownChecker {
    pub fn new(shutdown: Arc<AtomicBool>) -> Self {
        ShutdownChecker {
            shutdown,
            last_check_ts: 0,
        }
    }

    pub fn check(&mut self) -> bool {
        self.check_impl(false)
    }

    pub fn check_force(&mut self) -> bool {
        self.check_impl(true)
    }

    fn check_impl(&mut self, force: bool) -> bool {
        let cur_ts = time::cur_ts();
        if cur_ts <= self.last_check_ts || force {
            return self.shutdown.load(Ordering::Relaxed);
        }
        self.last_check_ts = cur_ts;
        return false;
    }
}
