use crate::helpers;
use crate::request::Request;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct Worker {
    num: i32,
    thread: Option<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

pub type WorkerPtr = Arc<RwLock<Worker>>;

impl Worker {
    pub fn new(num: i32, queue: crossbeam_channel::Receiver<Request>) -> WorkerPtr {
        let worker = Arc::new(RwLock::new(Worker {
            num,
            shutdown: Arc::new(AtomicBool::new(false)),
            thread: None,
        }));
        let worker_ref = worker.clone();
        let th = thread::spawn(move || worker_loop(worker_ref, queue));
        worker.write().unwrap().thread = Some(th);
        worker
    }
}

fn worker_loop(worker: WorkerPtr, queue: crossbeam_channel::Receiver<Request>) {
    let mut shutdown_checker =
        helpers::ShutdownChecker::new(worker.read().unwrap().shutdown.clone());
    loop {
        match queue.recv_timeout(Duration::from_millis(50)) {
            Ok(req) => {
                process(&worker, req);
            }
            Err(_) => {
                if shutdown_checker.check_force() {
                    return;
                }
            }
        }
        if shutdown_checker.check() {
            return;
        }
    }
}

fn process(worker: &WorkerPtr, req: Request) {
    log::debug!("worker {} got request", worker.read().unwrap().num);
    req.respond(&String::from(format!(
        "The number is {}",
        worker.read().unwrap().num
    )))
}

pub fn shutdown(worker: &WorkerPtr) {
    let mut th;
    let num;
    {
        let mut locked = worker.write().unwrap();
        locked.shutdown.store(true, Ordering::Relaxed);
        th = locked.thread.take();
        num = locked.num;
    }

    if let Some(handle) = th.take() {
        let msg = format!("fail to join worker with id={}", num);
        handle.join().expect(msg.as_str());
    }
}
