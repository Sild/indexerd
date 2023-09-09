extern crate actix_web;
extern crate tokio;
use actix_web::web::service;
use actix_web::{
    get, http::KeepAlive, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use std::ops::Deref;
use std::option::Option;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    thread::JoinHandle,
};

#[derive(Default)]
pub struct Server {
    admin_port: u16,
    user_port: u16,
    admin_service: Option<JoinHandle<()>>,
    user_service: Option<JoinHandle<()>>,
    shutdown_flag: AtomicBool,
}

impl Server {
    pub async fn new(admin_port: u16, user_port: u16) -> Self {
        Self {
            admin_port,
            user_port,
            admin_service: None,
            user_service: None,
            shutdown_flag: AtomicBool::new(false),
        }
    }
    // #[actix_web::main]
    // pub fn start(&mut self) {
    //
    // }
}

pub fn run(server: &Arc<Mutex<Server>>) -> std::io::Result<()> {
    run_admin_service(&server, "127.0.0.1:8089")?;
    init_engine(&server)?;
    run_user_service(&server, "127.0.0.1:8088")?;
    Ok(())
}

pub fn shutdown(server: &Arc<Mutex<Server>>) -> std::io::Result<()> {
    println!("shutdown server...");

    let mut user_serv: Option<JoinHandle<()>> = None;
    let mut admin_serv: Option<JoinHandle<()>> = None;
    {
        let mut locked = server.lock().unwrap();
        locked.shutdown_flag.store(true, Ordering::Relaxed);
        user_serv = locked.user_service.take();
        user_serv = locked.admin_service.take();
    }
    println!("waiting for user service to stop...");
    if let Some(handle) = user_serv.take() {
        handle
            .handle
            .join()
            .expect("fail to join user service thread");
    }
    println!("waiting for admin service to stop ...");
    if let Some(handle) = admin_serv.take() {
        handle.join().expect("fail to join admin service thread");
    }
    println!("server stopped");

    Ok(())
}

fn run_admin_service(server: &Arc<Mutex<Server>>, bind_addr: &str) -> std::io::Result<()> {
    println!("admin service (bind: {}) starting...", bind_addr);
    let local_server = server.clone();
    let listener = TcpListener::bind(bind_addr).expect("fail to bind admin service");
    let thread = thread::spawn(move || {
        for stream in listener.incoming() {
            match (stream) {
                Ok(stream) => {
                    if !handle_admin_connection(local_server.to_owned(), stream) {
                        return;
                    }
                }
                Err(e) => {
                    println!("connection failed {}", e);
                }
            }
        }
    });
    let mut locked_server = server.lock().unwrap();
    locked_server.admin_service = Some(thread);
    println!("admin service (bind: {}) started.", bind_addr);
    Ok(())
}

fn run_user_service(server: &Arc<Mutex<Server>>, bind_addr: &str) -> std::io::Result<()> {
    println!("user service (bind: {}) starting...", bind_addr);
    let mut local_server = server.clone();
    let listener = TcpListener::bind(bind_addr).unwrap();
    let thread = thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            if !handle_user_connection(local_server.to_owned(), stream) {
                return;
            }
        }
    });
    let mut locked_server = server.lock().unwrap();
    locked_server.user_service = Some(thread);
    println!("user service (bind {}): started.", bind_addr);
    Ok(())
}

fn init_engine(server: &Arc<Mutex<Server>>) -> std::io::Result<()> {
    println!("engine is starting...");

    println!("engine is ready");
    Ok(())
}

fn handle_admin_connection(mut server: Arc<Mutex<Server>>, mut stream: TcpStream) -> bool {
    if server.lock().unwrap().shutdown_flag.load(Ordering::Relaxed) {
        return false;
    }
    let response = "HTTP/1.1 200 OK\r\n\r\nHello admin";
    stream.write_all(response.as_bytes()).unwrap();
    true
}

fn handle_user_connection(server: Arc<Mutex<Server>>, mut stream: TcpStream) -> bool {
    println!("Got new user connection");
    if server.lock().unwrap().shutdown_flag.load(Ordering::Relaxed) {
        return false;
    }
    let response = "HTTP/1.1 200 OK\r\n\r\nHello user";
    stream.write_all(response.as_bytes()).unwrap();
    true
}
