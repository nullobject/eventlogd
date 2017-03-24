/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 */

extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate router;
extern crate rusqlite;
extern crate time;

use rusqlite::{Connection, Result};
use std::sync::mpsc::channel;

mod command;
mod entry;
mod request;
mod server;
mod uploader;

use command::Command::WriteEntry;
use entry::Entry;
use request::Request::WriteData;
use server::spawn_server;
use uploader::spawn_uploader;

fn create_entry(conn: &Connection, data: String) -> Entry {
    let timestamp = time::get_time();
    conn.execute("INSERT INTO entries (timestamp, data) VALUES (?, ?)", &[&timestamp, &data]).unwrap();
    let id = conn.last_insert_rowid();
    Entry { id: id, timestamp: timestamp, data: data }
}

fn run() -> Result<()> {
    info!("Starting up...");

    let conn = Connection::open("eventlogd.db").unwrap();
    let (server_tx, server_rx) = channel();
    let (uploader_tx, uploader_rx) = channel();

    spawn_server(server_tx).unwrap();
    spawn_uploader(uploader_rx).unwrap();

    loop {
        let request = server_rx.recv().unwrap();

        match request {
            WriteData(data) => {
                let entry = create_entry(&conn, data);
                uploader_tx.send(WriteEntry(entry)).unwrap();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    run().unwrap();
}
