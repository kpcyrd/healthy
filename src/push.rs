use crate::server::{StatusServer, Join, Leave, PushStatus};
use std::time::{Duration, Instant};

use actix::prelude::*;
// use actix_broker::BrokerSubscribe;
use actix_broker::BrokerIssue;
// use actix_web::{middleware, web, App, Result, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct MyWebSocket {
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.join(ctx);
        self.hb(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.issue_system_sync(Leave(self.id), ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        // println!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl MyWebSocket {
    pub fn new() -> Self {
        Self {
            id: 0,
            hb: Instant::now(),
        }
    }

    fn join(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        // First send a leave message for the current room
        // let leave_msg = LeaveRoom(self.room.clone(), self.id);
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        // self.issue_system_sync(leave_msg, ctx);
        // Then send a join message for the new room
        let join_msg = Join(
            ctx.address().recipient(),
        );

        StatusServer::from_registry()
            .send(join_msg)
            .into_actor(self)
            .then(|id, act, _ctx| {
                if let Ok(id) = id {
                    act.id = id;
                }

                fut::ok(())
            })
            .spawn(ctx);
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                info!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

impl Handler<PushStatus> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: PushStatus, ctx: &mut Self::Context) {
        let msg = serde_json::to_string(&msg).unwrap();
        ctx.text(msg);
    }
}
