#[macro_use]
extern crate log;

// use std::time::{Duration, Instant};

// use actix::prelude::*;
// use actix_broker::BrokerSubscribe;
use actix_web::{middleware, web, App, Result, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

pub mod push;
use push::MyWebSocket;
pub mod server;

fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    ws::start(MyWebSocket::new(), &r, stream)
}

fn style(_req: HttpRequest) -> Result<HttpResponse> {
    let body = std::fs::read_to_string("style.css").unwrap();
    // let body = include_str!("../style.css")
    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(body))
}

fn script(_req: HttpRequest) -> Result<HttpResponse> {
    let body = std::fs::read_to_string("script.js").unwrap();
    // let body = include_str!("../script.js")
    Ok(HttpResponse::Ok()
        .content_type("text/javascript; charset=utf-8")
        .body(body))
}
fn index(_req: HttpRequest) -> Result<HttpResponse> {
    let body = std::fs::read_to_string("index.html").unwrap();
    // let body = include_str!("../index.html")
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "healthy=info,actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(web::resource("/style.css").to(style))
            .service(web::resource("/script.js").to(script))
            .service(web::resource("/").to(index))
    })
    // start http server on 127.0.0.1:8080
    .bind("127.0.0.1:8080")?
    .run()
}
