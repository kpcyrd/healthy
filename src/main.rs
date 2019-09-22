#[macro_use] extern crate log;

use env_logger::Env;
use healthy::args::Args;
use healthy::config::Config;
use healthy::errors::*;
use healthy::ping;
use healthy::push::MyWebSocket;
use healthy::server::{PushStatus, Status};
use healthy::STATUS;
use std::thread;
use structopt::StructOpt;

use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

fn ping_loop(config: Config) {
    thread::spawn(move || {
        loop {
            debug!("pinging targets");

            let mut hosts = Vec::new();
            for target in &config.target {
                hosts.push(target.host);
            }

            if let Ok(status) = ping::send(&hosts) {
                debug!("updating status");
                let mut push = PushStatus::default();

                for target in &config.target {
                    if let Some(&healthy) = status.get(&target.host) {
                        push.entries.push(Status {
                            name: target.name.clone(),
                            healthy,
                        });
                        hosts.push(target.host);
                    }
                }

                let mut w = STATUS.write().unwrap();
                *w = push;
            }

            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

fn ws_index(r: HttpRequest, stream: web::Payload) -> actix_web::Result<HttpResponse> {
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

fn run() -> Result<()> {
    let args = Args::from_args();

    let env = match args.verbose {
        0 => "healthy=info,actix_server=info,actix_web=info",
        1 => "healthy=debug,actix_server=info,actix_web=info",
        _ => "debug",
    };

    env_logger::init_from_env(Env::default()
        .default_filter_or(env));

    let config = Config::load(&args.config)
        .expect("failed to load config");

    let bind = args.bind.unwrap_or_else(|| String::from("127.0.0.1:8080"));
    let server = HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(web::resource("/style.css").to(style))
            .service(web::resource("/script.js").to(script))
            .service(web::resource("/").to(index))
    })
    .bind(&bind)?;

    ping_loop(config);
    server.run()?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
