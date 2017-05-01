/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 *
 * TODO:
 *
 * - Queue requests until they are processed. e.g. The `WriteData` request shouldn't succeeed
 *   until it is journaled. A `GetRange` request shouldn't succeed until the data has been
 *   retrieved from S3.
 *
 * - Load the journal into the queue when the server restarts.
 *
 */

extern crate chrono;
extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate router;
extern crate rusqlite;

use rusqlite::{Connection, Result};
use std::ops::Range;
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
    conn.execute("INSERT INTO entries (data) VALUES (?)", &[&data]).unwrap();
    let id = conn.last_insert_rowid();
    conn.query_row("SELECT id, timestamp, data FROM entries WHERE id = ?", &[&id], |row| {
        Entry {
            id: row.get(0),
            timestamp: row.get(1),
            data: row.get(2)
        }
    }).unwrap()
}

fn delete_entries(conn: &Connection, range: Range<i64>) {
    conn.execute("DELETE FROM entries WHERE id >= ? AND id <= ?", &[&range.start, &range.end]).unwrap();
}

// FIXME: We can't rely on processing these messages continuously because the queues block while
// waiting.
fn run() -> Result<()> {
    info!("Starting up...");

    let conn = Connection::open("eventlogd.db").unwrap();
    let (server_out_tx, server_out_rx) = channel();
    let (uploader_in_tx, uploader_in_rx) = channel();
    let (uploader_out_tx, uploader_out_rx) = channel();

    spawn_server(server_out_tx).unwrap();
    spawn_uploader(uploader_in_rx, uploader_out_tx).unwrap();

    loop {
        let server_request = server_out_rx.try_recv();

        if server_request.is_ok() {
            let request = server_request.unwrap();

            match request {
                WriteData(data) => {
                    let entry = create_entry(&conn, data);
                    uploader_in_tx.send(WriteEntry(entry)).unwrap();
                }
                _ => {}
            }
        }

        let uploader_response = uploader_out_rx.try_recv();

        if uploader_response.is_ok() {
            let response = uploader_response.unwrap();
            trace!("3");

            match response {
                DeleteRange(range) => {
                    delete_entries(&conn, range);
                }
                _ => {}
            }
        }

        let ten_millis = std::time::Duration::from_millis(10);
        std::thread::sleep(ten_millis);
    }
}

fn main() {
    env_logger::init().unwrap();
    run().unwrap();
}
