extern crate crossbeam_channel;

use crate::request::Request;
use crate::store::store::Store;

use crate::worker;
use crate::worker::Worker;
use std::sync::Arc;

pub struct Engine {
    _store: Arc<Store>,
    workers: Vec<worker::WorkerPtr>,
}

impl Engine {
    pub fn new(http_queue: crossbeam_channel::Receiver<Request>) -> Self {
        let mut engine = Engine {
            _store: Arc::new(Store::default()),
            workers: Vec::new(),
        };

        for worker_num in 0..10 {
            let worker = Worker::new(worker_num, http_queue.clone());
            engine.workers.push(worker);
        }
        engine
    }

    pub fn _set_new_store(&mut self, store: Arc<Store>) {
        self._store = store;
    }

    // pub fn
    pub fn shutdown(&mut self) {
        println!("stopping engine...");
        for worker in self.workers.iter_mut() {
            worker::shutdown(worker);
        }
        println!("engine stopped");
    }
}
