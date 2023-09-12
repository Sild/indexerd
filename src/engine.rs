extern crate crossbeam_channel;
extern crate log;
use crate::request::Request;
use crate::store::store::Store;

use crate::worker;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub struct Engine {
    _store: Arc<Store>,
    shutdown: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
}

impl Engine {
    pub fn new(http_queue: crossbeam_channel::Receiver<Request>) -> Self {
        let mut engine = Engine {
            _store: Arc::new(Store::default()),
            shutdown: Arc::new(AtomicBool::new(false)),
            workers: Vec::new(),
        };

        for worker_num in 0..10 {
            let th = worker::run(worker_num, http_queue.clone(), engine.shutdown.clone());
            engine.workers.push(th);
        }
        engine
    }

    pub fn _set_new_store(&mut self, store: Arc<Store>) {
        self._store = store;
    }

    // pub fn
    pub fn shutdown(&mut self) {
        log::info!("stopping engine...");
        self.shutdown.store(true, Ordering::Relaxed);
        for th in self.workers.drain(..) {
            th.join().unwrap()
        }
        log::info!("engine stopped");
    }
}
