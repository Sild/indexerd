extern crate actix_web;
extern crate crossbeam_channel;
extern crate hwloc2;
extern crate libc;
extern crate tiny_http;
extern crate tokio;
use crate::task::Task;
use crate::{config, engine, helpers};
use crossbeam_channel::Sender;

use std::io::Error;

use crate::data::updater;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use std::{thread, thread::JoinHandle};

pub struct Server {
    conf: config::Server,
    admin_srv: JoinHandle<()>,
    user_srv: JoinHandle<()>,
    updater: updater::Updater,
}

impl Server {
    pub fn new(conf: &config::Server, shutdown: Arc<AtomicBool>) -> Result<Server, Error> {
        let (send_queue, rcv_queue) = crossbeam_channel::bounded(1000);
        let admin_srv = run_http_listener(
            shutdown.clone(),
            conf.service.admin_port,
            "admin_srv",
            &send_queue,
        )?;
        let user_srv = run_http_listener(
            shutdown.clone(),
            conf.service.user_port,
            "user_srv",
            &send_queue,
        )?;
        let engine = engine::Engine::new(&conf.engine, rcv_queue);
        let updater =
            updater::Updater::new(&conf.updater, engine, &shutdown).expect("fail to init updater");

        let server = Self {
            conf: conf.clone(),
            admin_srv,
            user_srv,
            updater,
        };

        Ok(server)
    }

    pub fn wait_shutdown(mut self) -> std::io::Result<()> {
        log::info!("waiting for shutdown...");

        self.user_srv.join().expect("fail join user_srv thread");
        log::info!("user service stopped");

        self.updater.shutdown();

        log::info!("waiting for admin service to stop ...");
        self.admin_srv.join().expect("fail join admin_srv thread");
        log::info!("admin service stopped");
        log::info!("app stopped");
        Ok(())
    }
}

fn run_http_listener(
    shutdown: Arc<AtomicBool>,
    port: u16,
    th_name: &str,
    send_queue: &Sender<Task>,
) -> std::io::Result<JoinHandle<()>> {
    let bind_addr = format!("0.0.0.0:{}", port);
    log::info!("starting {} thread (bind: {})...", th_name, bind_addr);

    let send_queue = send_queue.clone();
    let th = thread::Builder::new()
        .name(th_name.to_string())
        .spawn(move || {
            if let Err(err) = helpers::bind_thread(0) {
                log::error!(
                    "Fail to bind {} to core {} with err={:?}",
                    thread::current().name().unwrap_or("noname"),
                    0,
                    err
                );
            }
            let service = match tiny_http::Server::http(bind_addr.as_str()) {
                Ok(s) => s,
                _ => panic!("Fail to start service for bind={}", bind_addr),
            };
            service_loop(service, send_queue, shutdown);
        });

    log::info!("thread {} started", th_name);
    Ok(th.unwrap())
}

fn service_loop(service: tiny_http::Server, engine_queue: Sender<Task>, shutdown: Arc<AtomicBool>) {
    let mut shutdown_checker = helpers::ShutdownChecker::new(shutdown);
    loop {
        match service.recv_timeout(Duration::from_millis(50)) {
            Ok(Some(req)) => handle_connection(req, &engine_queue),
            _ => {
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

fn handle_connection(req: tiny_http::Request, queue: &Sender<Task>) {
    log::debug!(
        "received request! method: {:?}, url: {:?}, headers: {:?}",
        req.method(),
        req.url(),
        req.headers()
    );

    let task = Task::new(req);

    if let Err(e) = queue.send(task) {
        log::warn!("Fail to add request to queue with err={}", e)
    }
}
