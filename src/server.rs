/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 */

extern crate iron;

use self::iron::{Handler};
use self::iron::status;
use self::iron::prelude::*;
use std::io::prelude::*;
use std::io::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread;

use command::Command::{self, WriteEntry};
use entry::Entry;

struct ServerContext {
    id: AtomicUsize,
    tx: Mutex<Sender<Command>>
}

impl ServerContext {
    fn new(id: u32, tx: Sender<Command>) -> Self {
        ServerContext {
            id: AtomicUsize::new(id as usize),
            tx: Mutex::new(tx)
        }
    }

    fn increment_counter(&self) -> usize {
        self.id.fetch_add(1, Ordering::Relaxed)
    }
}

impl Handler for ServerContext {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut payload = String::new();
        req.body.read_to_string(&mut payload).unwrap();
        let old_id = self.increment_counter();
        let entry = Entry::new(old_id as u32, payload);
        self.tx.lock().unwrap().send(WriteEntry(entry)).unwrap();
        Ok(Response::with((status::Created, "")))
    }
}

// The server receives requests from clients, writes them to a journal, and pushes them into a
// queue. The journal allows the requests to be replayed into the queue if the server goes down.
fn server(tx: Sender<Command>, id: u32) -> Result<(), Error> {
    let context = ServerContext::new(id, tx);
    let router = router!(root: post "/" => context);
    Iron::new(router).http("localhost:6000").unwrap();
    Ok(())
}

pub fn spawn_server(tx: Sender<Command>, id: u32) -> Result<(), Error> {
    thread::spawn(move || {
        server(tx, id).unwrap();
    });

    Ok(())
}
