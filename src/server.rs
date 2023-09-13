extern crate actix_web;
extern crate crossbeam_channel;
extern crate tiny_http;
extern crate tokio;
use crate::request::Request;
use crate::{engine, helpers};
use crossbeam_channel::{Receiver, Sender};
use std::io::Error;
use std::option::Option;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{thread, thread::JoinHandle};

pub struct Server {
    admin_service: Option<JoinHandle<()>>,
    user_service: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    engine: engine::Engine,
}

impl Server {
    pub fn new(_admin_port: u16, _user_port: u16) -> Result<Server, Error> {
        let (task_snd_queue, task_rcv_queue): (Sender<Request>, Receiver<Request>) =
            crossbeam_channel::bounded(1000);

        let mut server = Self {
            admin_service: None,
            user_service: None,
            shutdown: Arc::new(AtomicBool::new(false)),
            engine: engine::Engine::new(task_rcv_queue),
        };

        let admin_service = server.run_service("0.0.0.0:8089", "admin", &task_snd_queue)?;
        server.admin_service = Some(admin_service);

        server.init_engine()?;

        let user_uservice = server.run_service("0.0.0.0:8088", "user", &task_snd_queue)?;
        server.user_service = Some(user_uservice);
        Ok(server)
    }

    pub fn shutdown(&mut self) -> std::io::Result<()> {
        log::info!("shutdown server...");

        self.shutdown.store(true, Ordering::Relaxed);
        log::info!("waiting for user service to stop...");
        if let Some(handle) = self.user_service.take() {
            handle.join().expect("fail to join user service thread");
        }

        self.engine.shutdown();

        log::info!("waiting for admin service to stop ...");
        if let Some(handle) = self.admin_service.take() {
            handle.join().expect("fail to join admin service thread");
        }
        log::info!("server stopped");

        Ok(())
    }

    fn init_engine(&self) -> std::io::Result<()> {
        log::info!("engine is starting...");

        log::info!("engine is ready");
        Ok(())
    }

    fn run_service(
        &self,
        bind_addr: &str,
        tag: &str,
        task_snd_queue: &Sender<Request>,
    ) -> std::io::Result<JoinHandle<()>> {
        log::info!("{} service (bind: {}) starting...", tag, bind_addr);

        let service = match tiny_http::Server::http(bind_addr) {
            Ok(s) => s,
            _ => panic!("Fail to start service for bind={}", bind_addr),
        };
        let engine_queue = task_snd_queue.clone();
        let shutdown = self.shutdown.clone();
        let th = thread::spawn(move || service_loop(service, engine_queue, shutdown));
        log::info!("{} service (bind: {}) started.", tag, bind_addr);
        Ok(th)
    }
}

fn service_loop(
    service: tiny_http::Server,
    engine_queue: Sender<Request>,
    shutdown: Arc<AtomicBool>,
) {
    let mut shutdown_checker = helpers::ShutdownChecker::new(shutdown.clone());
    loop {
        match service.recv_timeout(Duration::from_millis(50)) {
            Ok(Some(req)) => handle_connection(req, &engine_queue),
            _ => {
                if shutdown_checker.check_force() {
                    return;
                }
            }
        }
        if shutdown_checker.check_force() {
            return;
        }
    }
}

fn handle_connection(req: tiny_http::Request, queue: &Sender<Request>) {
    log::debug!(
        "received request! method: {:?}, url: {:?}, headers: {:?}",
        req.method(),
        req.url(),
        req.headers()
    );

    if let Err(e) = queue.send(Request::new(req)) {
        log::warn!("Fail to add request to queue with err={}", e)
    }
}
