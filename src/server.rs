/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 */

extern crate iron;

use self::iron::{Handler, Iron};
use self::iron::status;
use std::io::prelude::*;
use std::io::Error;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread;

use request::Request::{self, WriteData};

struct ServerContext {
    tx: Mutex<Sender<Request>>
}

impl ServerContext {
    fn new(tx: Sender<Request>) -> Self {
        ServerContext {
            tx: Mutex::new(tx)
        }
    }
}

impl Handler for ServerContext {
    fn handle(&self, req: &mut iron::request::Request) -> iron::IronResult<iron::response::Response> {
        let mut data = String::new();
        req.body.read_to_string(&mut data).unwrap();
        self.tx.lock().unwrap().send(WriteData(data)).unwrap();
        Ok(iron::response::Response::with((status::Created, "")))
    }
}

// The server receives requests from clients, writes them to a journal, and pushes them into a
// queue. The journal allows the requests to be replayed into the queue if the server goes down.
fn server(tx: Sender<Request>) -> Result<(), Error> {
    let context = ServerContext::new(tx);
    let router = router!(root: post "/" => context);
    Iron::new(router).http("localhost:6000").unwrap();
    Ok(())
}

pub fn spawn_server(tx: Sender<Request>) -> Result<(), Error> {
    thread::spawn(move || {
        server(tx).unwrap();
    });

    Ok(())
}
