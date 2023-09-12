use crate::helpers;
use crate::request::Request;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct WorkerData {
    num: i32,
    task_queue: crossbeam_channel::Receiver<Request>,
}

pub fn run(
    num: i32,
    task_queue: crossbeam_channel::Receiver<Request>,
    shutdown: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let worker_data = WorkerData { num, task_queue };
    thread::spawn(move || worker_loop(worker_data, shutdown))
}

fn worker_loop(mut worker_data: WorkerData, shutdown: Arc<AtomicBool>) {
    let mut sd_checker = helpers::ShutdownChecker::new(shutdown);
    loop {
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
