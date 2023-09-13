use crate::helpers;
use crate::request::Request;
use crate::store::store;
use crossbeam_channel::Receiver;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub type ControlTask = Box<dyn FnOnce(&mut WorkerData) + Send + 'static>;
// pub trait ControlTask {
//     pub fn (&mut WorkerData);
// }

pub struct WorkerData {
    pub num: i32,
    pub task_queue: Receiver<Request>,
    pub ctl_task_queue: Receiver<ControlTask>,
    pub store: Arc<RwLock<store::Store>>,
}

pub fn run(worker_data: WorkerData, shutdown: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn(move || worker_loop(worker_data, shutdown))
}

fn worker_loop(mut worker_data: WorkerData, shutdown: Arc<AtomicBool>) {
    let mut sd_checker = helpers::ShutdownChecker::new(shutdown);
    loop {
        if let Ok(ctl_task) = worker_data.ctl_task_queue.try_recv() {
            ctl_task(&mut worker_data)
        }
        match worker_data
            .task_queue
            .recv_timeout(Duration::from_millis(50))
        {
            Ok(req) => {
                process(&mut worker_data, req);
            }
            Err(_) => {
                if sd_checker.check_force() {
                    break;
                }
            }
        }
        if sd_checker.check() {
            break;
        }
    }
    log::info!("worker {} stopped", worker_data.num)
}

fn process(worker_data: &WorkerData, req: Request) {
    log::debug!("worker {} got request", worker_data.num);
    req.respond(&format!("The number is {}", worker_data.num))
}
