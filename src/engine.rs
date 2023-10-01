extern crate crossbeam_channel;
extern crate log;
use crate::data::store::Store;
use crate::task::Task;

use crate::worker::{ControlTask, WorkerData};
use crate::{config, worker};
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

pub struct Engine {
    store: Arc<RwLock<Store>>,
    shutdown_workers: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
    ctl_queues: Vec<Sender<ControlTask>>,
    conf: config::Engine,
}

impl Engine {
    pub fn new(conf: &config::Engine, task_queue_rcv: Receiver<Task>) -> Self {
        let mut engine = Engine {
            store: Arc::new(RwLock::new(Store::default())),
            shutdown_workers: Arc::new(AtomicBool::new(false)),
            workers: Vec::new(),
            ctl_queues: Vec::new(),
            conf: conf.clone(),
        };

        for worker_num in 0..=2 {
            let (ctl_queue_snd, ctl_queue_rcv): (Sender<ControlTask>, Receiver<ControlTask>) =
                crossbeam_channel::bounded(1000);

            let worker_data = WorkerData {
                num: worker_num,
                task_queue: task_queue_rcv.clone(),
                ctl_task_queue: ctl_queue_rcv,
                store: engine.store.clone(),
                config: engine.conf.worker,
            };

            let th = worker::run(worker_data, engine.shutdown_workers.clone());
            engine.workers.push(th);
            engine.ctl_queues.push(ctl_queue_snd);
            log::info!("worker {} started", worker_num);
        }
        engine
    }

    pub fn set_new_store(&mut self, store: &Arc<RwLock<Store>>) {
        self.store = store.clone();
        for queue in self.ctl_queues.iter_mut() {
            let store_copy = self.store.clone();
            let func = move |worker_data: &mut WorkerData| {
                worker_data.store = store_copy;
            };
            if let Err(e) = queue.send(Box::new(func)) {
                log::warn!("Fail to add task to ctl_queue: {}", e);
            }
        }
    }

    pub fn update_config(&mut self, conf: config::Engine) {
        self.conf = conf;
        for queue in self.ctl_queues.iter_mut() {
            let conf_copy = self.conf.worker;
            let func = move |worker_data: &mut WorkerData| {
                worker_data.config = conf_copy;
            };
            if let Err(e) = queue.send(Box::new(func)) {
                log::warn!("Fail to add task to ctl_queue: {:?}", e);
            }
        }
    }

    pub fn stop(&mut self) {
        log::info!("stopping engine...");
        self.shutdown_workers.store(true, Ordering::Relaxed);
        for th in self.workers.drain(..) {
            th.join().unwrap()
        }
        log::info!("engine stopped");
    }
}
