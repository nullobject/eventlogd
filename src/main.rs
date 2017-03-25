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

mod core;
mod server;
mod uploader;

use core::Command::WriteEntry;
use core::Entry;
use core::Request::WriteData;
use core::Response::DeleteRange;
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
    let (server_out_tx, server_out_rx) = channel();
    let (uploader_in_tx, uploader_in_rx) = channel();
    let (uploader_out_tx, uploader_out_rx) = channel();

    spawn_server(server_out_tx).unwrap();
    spawn_uploader(uploader_in_rx, uploader_out_tx).unwrap();

    loop {
        let request = server_out_rx.recv().unwrap();

        match request {
            WriteData(data) => {
                let entry = create_entry(&conn, data);
                uploader_in_tx.send(WriteEntry(entry)).unwrap();
            }
        }

        let response = uploader_out_rx.try_recv().unwrap();

        match response {
            // TODO: Delete the entries from the DB.
            DeleteRange(range) => {}
            _ => {}
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    run().unwrap();
}
