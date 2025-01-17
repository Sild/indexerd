extern crate crossbeam_channel;
extern crate hwloc2;
extern crate libc;
extern crate tiny_http;
use crate::task::HttpTask;
use crate::{config, helpers};
use crossbeam_channel::Sender;

use std::io::Error;

use crate::data::updater::{Updater, UpdaterPtr};
use crate::engine::Engine;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::Duration;
use std::{thread, thread::JoinHandle};

pub struct Server {
    #[allow(dead_code)]
    conf: config::Server,
    http_srv: JoinHandle<()>,
    engine: Arc<RwLock<Engine>>,
    updater: UpdaterPtr,
    stop_flag: Arc<AtomicBool>,
    wait_pair: Arc<(Mutex<bool>, Condvar)>,
}

impl Server {
    pub fn new(
        conf: &config::Server,
        wait_pair: Arc<(Mutex<bool>, Condvar)>,
    ) -> Result<Server, Error> {
        let (send_queue, rcv_queue) = crossbeam_channel::bounded(1000);
        let stop_flag = Arc::new(AtomicBool::new(false));

        let http_srv = run_http_listener(
            stop_flag.clone(),
            conf.service.listening_port,
            "http_srv",
            &send_queue,
        )?;

        let engine = Arc::new(RwLock::new(Engine::new(&conf.engine, rcv_queue)));
        let updater = Updater::new(&conf.updater, engine.clone()).unwrap();

        let server = Self {
            conf: conf.clone(),
            http_srv,
            engine,
            updater,
            stop_flag,
            wait_pair,
        };

        Ok(server)
    }

    pub fn wait_stop(self) -> std::io::Result<()> {
        let (lock, cvar) = &*self.wait_pair;
        let mut working = lock.lock().unwrap();
        while *working {
            working = cvar.wait(working).unwrap();
        }

        log::info!("stop signal received, shutting down...");
        let mut updater_locked = self.updater.write().unwrap();
        updater_locked.stop();

        log::info!("stopping services...");
        self.stop_flag.store(true, Ordering::Release);

        let mut engine_locked = self.engine.write().unwrap();
        engine_locked.stop();

        self.http_srv.join().expect("fail join admin_srv thread");
        log::info!("admin service stopped");

        log::info!("app stopped");
        Ok(())
    }
}

fn run_http_listener(
    shutdown: Arc<AtomicBool>,
    port: u16,
    th_name: &str,
    send_queue: &Sender<HttpTask>,
) -> Result<JoinHandle<()>, Error> {
    let bind_addr = format!("127.0.0.1:{}", port);
    log::info!(
        "thread {} (bind: http://{}) starting...",
        th_name,
        bind_addr
    );

    let send_queue = send_queue.clone();
    let th_builder = thread::Builder::new().name(th_name.to_string());
    let th = th_builder.spawn(move || {
        helpers::bind_thread(0);
        let service = match tiny_http::Server::http(bind_addr.as_str()) {
            Ok(s) => s,
            _ => panic!("Fail to start service for bind={}", bind_addr),
        };
        service_loop(service, send_queue, shutdown);
    })?;

    log::info!("thread {} started", th_name);
    Ok(th)
}

fn service_loop(
    service: tiny_http::Server,
    engine_queue: Sender<HttpTask>,
    shutdown: Arc<AtomicBool>,
) {
    let mut stop_checker = helpers::StopChecker::new(shutdown);
    while !stop_checker.is_time() {
        if let Ok(Some(req)) = service.recv_timeout(Duration::from_millis(50)) {
            handle_connection(req, &engine_queue);
        }
    }
    log::info!(
        "thread {} stopped",
        thread::current().name().unwrap_or("noname")
    );
}

fn handle_connection(req: tiny_http::Request, queue: &Sender<HttpTask>) {
    log::trace!(
        "received request! method: {:?}, url: {:?}, headers: {:?}",
        req.method(),
        req.url(),
        req.headers()
    );

    let task = HttpTask::new(req);

    if let Err(e) = queue.send(task) {
        log::warn!("Fail to add request to queue with err={}", e)
    }
}
