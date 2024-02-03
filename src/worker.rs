use crate::data::store::Store;
use crate::handlers::{admin, search};
use crate::task::{AdminTask, HttpTask, SearchTask};
use crate::{config, helpers};
use crossbeam_channel::Receiver;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub type ControlTask = Box<dyn FnOnce(&mut WorkerData) + Send + 'static>;

pub struct WorkerData {
    pub num: i32,
    pub task_queue: Receiver<HttpTask>,
    pub ctl_task_queue: Receiver<ControlTask>,
    pub store: Arc<RwLock<Store>>,
    pub config: config::Worker,
}

pub fn run(worker_data: WorkerData, shutdown: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("worker_{}", worker_data.num))
        .spawn(move || {
            helpers::bind_thread((worker_data.num + 2) as usize);
            worker_loop(worker_data, shutdown)
        })
        .unwrap()
}

fn worker_loop(mut worker_data: WorkerData, shutdown: Arc<AtomicBool>) {
    let mut stop_checker = helpers::StopChecker::new(shutdown);
    loop {
        if let Ok(ctl_task) = worker_data.ctl_task_queue.try_recv() {
            ctl_task(&mut worker_data)
        }
        match worker_data
            .task_queue
            .recv_timeout(Duration::from_millis(50))
        {
            Ok(req) => {
                process(&worker_data, req);
            }
            Err(_) => {
                if stop_checker.is_time_force() {
                    break;
                }
            }
        }
        if stop_checker.is_time() {
            break;
        }
    }
    log::info!("worker {} stopped", worker_data.num)
}

fn process(worker_data: &WorkerData, http_task: HttpTask) {
    log::trace!(
        "worker {} got request: {}",
        worker_data.num,
        http_task.url()
    );
    let store_r = worker_data.store.read().unwrap();

    let req_url = http_task.url();
    let store_ref = store_r.deref();
    let config = &worker_data.config;

    if req_url.starts_with("/admin") {
        let task = AdminTask::new(http_task, store_ref, config);
        admin::handle(task);
    } else if req_url.starts_with("/search") {
        let task = SearchTask::new(http_task, store_ref, config);
        search::handle(task);
    } else {
        http_task.respond_html("unknown method")
    }
}
