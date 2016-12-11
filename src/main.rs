extern crate rustc_serialize;
extern crate time;
extern crate zmq;

use rustc_serialize::json::{self};
use std::cmp;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, LineWriter, SeekFrom, Error};
use std::io::prelude::*;
use std::thread;

#[derive(RustcDecodable, RustcEncodable)]
struct Entry {
	id: u32,
	timestamp: String,
	payload: String
}

// Return the next ID in the journal.
fn get_next_id() -> Result<u32, Error> {
    let file = try!(File::open("journal.txt"));
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let mut id = 0;

    while try!(reader.read_line(&mut buffer)) > 0 {
        let entry: Entry = json::decode(&buffer).unwrap();
        id = cmp::max(entry.id, id);
        buffer.clear();
    }

    Ok(id + 1)
}

// Write journal entry.
fn write_entry<W: Write>(writer: &mut LineWriter<W>, id: u32, payload: String) -> Result<(), Error> {
    let timestamp = time::strftime("%Y-%m-%d %H:%M:%S.%f", &time::now()).unwrap();

    let entry = Entry {
        id: id,
        timestamp: timestamp,
        payload: payload
    };

    writeln!(writer, "{}", json::encode(&entry).unwrap())
}

// Read entries from journal and post events to S3.
fn reader_thread() -> Result<(), Error> {
    let mut file = try!(File::open("journal.txt"));

    // Seek to the end of the file.
    try!(file.seek(SeekFrom::End(0)));

    let mut reader = BufReader::new(file);
    let mut buffer = String::new();

    loop {
        while try!(reader.read_line(&mut buffer)) > 0 {
            print!("{}", buffer);
            buffer.clear();
        }
    }
}

fn journal_thread(mut id: u32) -> Result<(), Error> {
    let mut options = OpenOptions::new();
    let file = try!(options.append(true).open("journal.txt"));
    let mut writer = LineWriter::new(file);

    let context = zmq::Context::new();
    let broker = context.socket(zmq::REP).unwrap();

    assert!(broker.bind("tcp://*:5678").is_ok());

    loop {
        let payload = broker.recv_string(0).unwrap().unwrap();
        try!(write_entry(&mut writer, id, payload));
        broker.send(b"", 0).unwrap();
        id += 1;
    }
}

fn main() {
    let id = get_next_id().unwrap();

    thread::spawn(|| {
        reader_thread().unwrap();
    });

    journal_thread(id).unwrap();
}
