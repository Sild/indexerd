extern crate rand;

use crate::data::store::Store;
use crate::handlers::{admin, search};
use crate::task::{AdminTask, HttpTask, SearchTask, TaskContext};
use crate::{config, helpers};
use crossbeam_channel::Receiver;
use rand::Rng;
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
            helpers::bind_thread((2/*worker_data.num + 2*/) as usize);
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
                process(&mut worker_data, req);
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
    log::debug!(
        "worker {} got request: {}",
        worker_data.num,
        http_task.raw_req.url()
    );
    let store_r = worker_data.store.read().unwrap();

    let req_url = http_task.raw_req.url();
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
    // let res = 1; //mock_task(&worker_task);
    // worker_task
    //     .http_task
    //     .respond(&format!("worker_num={}, result={}", worker_data.num, res))
}

fn mock_task(worker_task: &AdminTask) -> String {
    let mut rng = rand::thread_rng();
    let matrix_size = 5;

    let mut matrix_a = Vec::new();
    for i in 0..matrix_size {
        matrix_a.push(vec![]);
        for _ in 0..matrix_size {
            matrix_a[i].push(rng.gen_range(0.0..999.0));
        }
    }

    let mut matrix_b = Vec::new();
    for i in 0..matrix_size {
        matrix_b.push(vec![]);
        for _ in 0..matrix_size {
            matrix_b[i].push(rng.gen_range(0.0..999.0));
        }
    }
    let res = matrix_multiply(&matrix_a, &matrix_b);

    let mut answer = String::new();
    for row in res.iter() {
        for elem in row.iter() {
            let val = match worker_task.context.config.need_multi {
                true => elem * 2.0,
                false => *elem,
            };
            answer += &format!("{},", val);
        }
    }
    answer
}

fn matrix_multiply(a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut result = Vec::new();

    for i in 0..a.len() {
        let mut row = Vec::new();
        for j in 0..b[0].len() {
            let mut cell = 0.0;
            for k in 0..a[0].len() {
                cell += a[i][k] * b[k][j];
            }
            row.push(cell);
        }
        result.push(row);
    }
    result
}
