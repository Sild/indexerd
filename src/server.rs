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
        let (engine_req_transfer, engine_req_receiver): (Sender<Request>, Receiver<Request>) =
            crossbeam_channel::bounded(1000);

        let mut server = Self {
            admin_service: None,
            user_service: None,
            shutdown: Arc::new(AtomicBool::new(false)),
            engine: engine::Engine::new(engine_req_receiver),
        };

        let admin_service = server.run_service("127.0.0.1:8089", "admin", &engine_req_transfer)?;
        server.admin_service = Some(admin_service);

        server.init_engine()?;

        let user_uservice = server.run_service("127.0.0.1:8088", "user", &engine_req_transfer)?;
        server.user_service = Some(user_uservice);
        Ok(server)
    }

    pub fn shutdown(&mut self) -> std::io::Result<()> {
        println!("shutdown server...");

        self.shutdown.store(true, Ordering::Relaxed);
        println!("waiting for user service to stop...");
        if let Some(handle) = self.user_service.take() {
            handle.join().expect("fail to join user service thread");
        }

        self.engine.shutdown();

        println!("waiting for admin service to stop ...");
        if let Some(handle) = self.admin_service.take() {
            handle.join().expect("fail to join admin service thread");
        }
        println!("server stopped");

        Ok(())
    }

    fn init_engine(&self) -> std::io::Result<()> {
        println!("engine is starting...");

        println!("engine is ready");
        Ok(())
    }

    fn run_service(
        &self,
        bind_addr: &str,
        tag: &str,
        engine_queue: &Sender<Request>,
    ) -> std::io::Result<JoinHandle<()>> {
        println!("{} service (bind: {}) starting...", tag, bind_addr);

        let service = match tiny_http::Server::http(bind_addr) {
            Ok(s) => s,
            _ => panic!("Fail to start sevice for bind={}", bind_addr),
        };
        let engine_queue = engine_queue.clone();
        let shutdown = self.shutdown.clone();
        let th = thread::spawn(move || service_loop(service, engine_queue, shutdown));
        println!("{} service (bind: {}) started.", tag, bind_addr);
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
    println!(
        "received request! method: {:?}, url: {:?}, headers: {:?}",
        req.method(),
        req.url(),
        req.headers()
    );

    match queue.send(Request::new(req)) {
        Err(e) => {
            println!("Fail to add request to queue with err={}", e)
        }
        _ => {}
    }
}
