extern crate tokio;
extern crate actix_web;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, http::KeepAlive};
use actix_web::cookie::time::Error;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Default)]
pub struct Server {
    // admin_service: actix_web::HttpServer::Server,
    // user_service: Server,
}

impl Server {
    // pub fn new()->Self {
    //     Self
    // }
    //
    pub fn run_user_service(&self, port: u16) -> std::io::Result<()> {
        println!("user service (port {}): starting...", port);
        let res = self.run_user_service_async(port);
        println!("user service (port {}): started.", port);
        Ok(())
    }

    pub fn run_admin_service(&self, port: u16) -> std::io::Result<()> {
        println!("admin service (port {}): starting...", port);
        let res = self.run_admin_service_async(port);
        println!("admin service (port {}): started...", port);
        Ok(())
    }

    #[actix_web::main]
    async fn run_admin_service_async(&self, port: u16) -> std::io::Result<()> {
        let service = HttpServer::new(|| {
            App::new()
                .service(hello)
                .route("/hey", web::get().to(manual_hello))
        }).keep_alive(KeepAlive::Os)
            .bind(("127.0.0.1", port))?
            .run();

        let _ = tokio::task::spawn(service);
        Ok(())
    }

    async fn run_user_service_async(&self, port: u16) -> std::io::Result<()> {
        println!("user service (port {}): starting...", port);
        let service = HttpServer::new(|| {
            App::new()
                .service(hello)
        }).keep_alive(KeepAlive::Os)
            .bind(("127.0.0.1", port))?
            .run();
        let _ = tokio::task::spawn(service);
        Ok(())
    }

    pub fn init_engine(&self) {
        println!("engine starting...");

        println!("engine ready");

    }
    //
    // #[get("/")]
    // async fn hello() -> impl Responder {
    //     HttpResponse::Ok().body("Hello world!")
    // }
    //
    // #[post("/echo")]
    // async fn echo(req_body: String) -> impl Responder {
    //     HttpResponse::Ok().body(req_body)
    // }
    //
    // async fn manual_hello(&self) -> impl Responder {
    //     HttpResponse::Ok().body("Hey there!")
    // }
}