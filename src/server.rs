use crate::STATUS;
use std::collections::HashMap;
use std::time::Duration;
use std::mem;
use serde_derive::{Serialize, Deserialize};

use actix::prelude::*;
use actix_broker::BrokerSubscribe;

const HEALTHCHECK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, PartialEq, Default, Message, Serialize, Deserialize)]
pub struct PushStatus {
    pub entries: Vec<Status>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub name: String,
    pub healthy: bool,
}

#[derive(Clone, Message)]
#[rtype(result = "usize")]
pub struct Join(pub Recipient<PushStatus>);

#[derive(Clone, Message)]
pub struct Leave(pub usize);

type Client = Recipient<PushStatus>;

#[derive(Default)]
pub struct StatusServer {
    clients: HashMap<usize, Client>,
    last_msg: Option<PushStatus>,
}

impl Actor for StatusServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<Leave>(ctx);
        // self.subscribe_system_async::<PushStatus>(ctx);
        self.hb(ctx);
    }
}

impl StatusServer {
    fn join_client(&mut self, client: Client) -> usize {
        let mut id = rand::random::<usize>();
        loop {
            if self.clients.contains_key(&id) {
                id = rand::random::<usize>();
            } else {
                break;
            }
        }

        if let Some(last_msg) = &self.last_msg {
            client.do_send(last_msg.clone()).ok();
        }

        self.clients.insert(id, client);
        info!("client joined ({} total clients)", self.clients.len());

        id
    }

    fn leave_client(&mut self, client: usize) {
        self.clients.remove(&client);
        info!("client left ({} total clients)", self.clients.len());
    }

    fn push_status(&mut self, msg: PushStatus) {
        if Some(&msg) == self.last_msg.as_ref() {
            debug!("status didn't change, not pushing an update");
            return;
        }
        info!("pushing an update to clients");
        self.last_msg = Some(msg.clone());

        let mut failed = 0;
        let mut clients = mem::replace(&mut self.clients, HashMap::new());
        for (id, client) in clients.drain() {
            if client.do_send(msg.clone()).is_ok() {
                self.clients.insert(id, client);
            } else {
                failed += 1;
            }
        }

        if failed > 0 {
            info!("{} client failed ({} remaining clients)", failed, self.clients.len());
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEALTHCHECK_INTERVAL, |act, _ctx| {
            let push = STATUS.read().unwrap().clone();
            act.push_status(push);
        });
    }
}

impl Handler<Join> for StatusServer {
    type Result = MessageResult<Join>;

    fn handle(&mut self, msg: Join, _ctx: &mut Self::Context) -> Self::Result {
        let id = self.join_client(msg.0);
        MessageResult(id)
    }
}

impl Handler<Leave> for StatusServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, _ctx: &mut Self::Context) {
        self.leave_client(msg.0);
    }
}

// TODO: maybe drop this
/*
impl Handler<PushStatus> for StatusServer {
    type Result = ();

    fn handle(&mut self, msg: PushStatus, _ctx: &mut Self::Context) {
        self.push_status(msg);
    }
}
*/

impl SystemService for StatusServer {}
impl Supervised for StatusServer {}
