extern crate rand;
use crate::helpers;
use crate::request::Request;
use crate::store::store;
use crossbeam_channel::Receiver;
use rand::Rng;
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
    thread::Builder::new()
        .name(format!("worker_{}", worker_data.num))
        .spawn(move || {
            if let Err(e) = helpers::bind_thread((worker_data.num + 1) as usize) {
                log::error!("bind thread failed: {:?}", e);
                return;
            }
            worker_loop(worker_data, shutdown)
        })
        .unwrap()
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
    let res = mock_task(worker_data);
    req.respond(&format!(
        "The number is {}, result={}",
        worker_data.num, res
    ))
}

fn mock_task(_worker_data: &WorkerData) -> String {
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
            answer += &format!("{},", elem);
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
