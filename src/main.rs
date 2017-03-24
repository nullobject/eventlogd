/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 */

extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate router;
extern crate rustc_serialize;

use rustc_serialize::json::{self};
use std::cmp;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, LineWriter, Error};
use std::io::prelude::*;
use std::sync::mpsc::channel;

mod command;
mod entry;
mod server;
mod uploader;

use command::Command::WriteEntry;
use entry::Entry;
use server::spawn_server;
use uploader::spawn_uploader;

// Return the next ID in the journal.
fn get_next_id() -> Result<u32, Error> {
    let file = File::open("journal.txt")?;
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let mut id = 0;

    while reader.read_line(&mut buffer)? > 0 {
        let entry: Entry = json::decode(&buffer).unwrap();
        id = cmp::max(entry.id, id);
        buffer.clear();
    }

    Ok(id + 1)
}

// Write a journal entry using the given writer.
fn write_entry_to_journal<W: Write>(writer: &mut LineWriter<W>, entry: &Entry) -> Result<(), Error> {
    writeln!(writer, "{}", json::encode(entry).unwrap())
}

fn run() -> Result<(), Error> {
    info!("Starting up...");

    let mut options = OpenOptions::new();
    let file = options.append(true).open("journal.txt")?;
    let mut writer = LineWriter::new(file);
    let next_id = get_next_id()?;
    let (server_tx, server_rx) = channel();
    let (uploader_tx, uploader_rx) = channel();

    spawn_server(server_tx, next_id)?;
    spawn_uploader(uploader_rx)?;

    // Receive message from server, journal it, and send to uploader.
    loop {
        let command = server_rx.recv().unwrap();
        match command {
            WriteEntry(entry) => {
                write_entry_to_journal(&mut writer, &entry)?;
                uploader_tx.send(entry).unwrap();
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    run().unwrap();
}
