extern crate crossbeam_channel;
extern crate log;
use crate::data::store::Store;
use crate::task::Task;

use crate::worker;
use crate::worker::{ControlTask, WorkerData};
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

pub struct Engine {
    store: Arc<RwLock<Store>>,
    shutdown_workers: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
    ctl_queues: Vec<Sender<ControlTask>>,
}

impl Engine {
    pub fn new(task_queue_rcv: Receiver<Task>) -> Self {
        let mut engine = Engine {
            store: Arc::new(RwLock::new(Store::default())),
            shutdown_workers: Arc::new(AtomicBool::new(false)),
            workers: Vec::new(),
            ctl_queues: Vec::new(),
        };

        for worker_num in 0..=2 {
            let (ctl_queue_snd, ctl_queue_rcv): (Sender<ControlTask>, Receiver<ControlTask>) =
                crossbeam_channel::bounded(1000);

            let worker_data = WorkerData {
                num: worker_num,
                task_queue: task_queue_rcv.clone(),
                ctl_task_queue: ctl_queue_rcv,
                store: engine.store.clone(),
            };

            let th = worker::run(worker_data, engine.shutdown_workers.clone());
            engine.workers.push(th);
            engine.ctl_queues.push(ctl_queue_snd);
            log::info!("worker {} started", worker_num);
        }
        engine
    }

    pub fn _set_new_store(&mut self, store: Arc<RwLock<Store>>) {
        self.store = store;
        for queue in self.ctl_queues.iter_mut() {
            let store_copy = self.store.clone();
            let func = move |worker_data: &mut WorkerData| {
                worker_data.store = store_copy;
            };
            _ = queue.send(Box::new(func));
        }
    }

    // pub fn
    pub fn shutdown(&mut self) {
        log::info!("stopping engine...");
        self.shutdown_workers.store(true, Ordering::Relaxed);
        for th in self.workers.drain(..) {
            th.join().unwrap()
        }
        log::info!("engine stopped");
    }
}
