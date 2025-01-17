extern crate crossbeam_channel;
extern crate log;

use crate::data::store::Store;
use crate::task::HttpTask;

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
    pub fn new(conf: &config::Engine, task_queue_rcv: Receiver<HttpTask>) -> Self {
        let mut engine = Engine {
            store: Arc::new(RwLock::new(Store::default())),
            shutdown_workers: Arc::new(AtomicBool::new(false)),
            workers: Vec::new(),
            ctl_queues: Vec::new(),
            conf: conf.clone(),
        };

        for worker_num in 0..=1 {
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
        log::info!(
            "set_new_store: {}",
            store.read().unwrap().get_store_stat().iteration
        );
        self.store = store.clone();
        // let counter = Arc::new(AtomicUsize::new(0));
        for queue in self.ctl_queues.iter_mut() {
            let store_copy = self.store.clone();
            // let counter_copy = counter.clone();
            let func = move |worker_data: &mut WorkerData| {
                worker_data.store = store_copy;
                // counter_copy.store(
                //     counter_copy.load(Ordering::Relaxed).add(1),
                //     Ordering::Relaxed,
                // );
            };
            if let Err(e) = queue.send(Box::new(func)) {
                log::warn!("Fail to add task to ctl_queue: {}", e);
            }
        }
        // wait until all workers get new store
        // while counter.load(Ordering::Relaxed) < self.workers.len() {}
    }

    #[allow(dead_code)]
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
