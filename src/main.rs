extern crate rustc_serialize;
extern crate time;
extern crate zmq;

use rustc_serialize::json::{self};
use std::cmp;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, LineWriter, BufRead, Write};
use std::thread;
use zmq::SNDMORE;

#[derive(RustcDecodable, RustcEncodable)]
struct Entry {
	id: u32,
	timestamp: String,
	payload: String
}

fn get_next_id() -> u32 {
    let fin = File::open("journal.txt").unwrap();
    let mut reader = BufReader::new(fin);
    let mut buffer = String::new();
    let mut max_id = 0;

    while reader.read_line(&mut buffer).unwrap() > 0 {
        let entry: Entry = json::decode(&buffer).unwrap();
        max_id = cmp::max(entry.id, max_id);
        buffer.clear();
    }

    return max_id + 1;
}

fn main() {
    let mut id = get_next_id();

    let mut options = OpenOptions::new();
    let fout = options.append(true).open("journal.txt").unwrap();
    let mut writer = LineWriter::new(fout);

    let context = zmq::Context::new();
    let broker = context.socket(zmq::ROUTER).unwrap();

    assert!(broker.bind("tcp://*:5678").is_ok());

    thread::spawn(|| {
        let fin = File::open("journal.txt").unwrap();
        let mut reader = BufReader::new(fin);
        let mut buffer = String::new();

        loop {
            while reader.read_line(&mut buffer).unwrap() > 0 {
                print!("{}", buffer);
                buffer.clear();
            }
        };
    });

    loop {
        let identity = broker.recv_bytes(0).unwrap();
        println!("identity: {:?}", identity);
        broker.send(&identity, SNDMORE).unwrap();

        broker.recv_bytes(0).unwrap();  // envelope delimiter

        let payload = broker.recv_string(0).unwrap().unwrap();
        println!("payload: {:?}", payload);
        broker.send(b"", SNDMORE).unwrap();

        let timestamp = time::strftime("%Y-%m-%d %H:%M:%S.%f", &time::now()).unwrap();
        let entry = Entry {
            id: id,
            timestamp: timestamp,
            payload: payload
        };

        write!(writer, "{}\n", json::encode(&entry).unwrap()).unwrap();

        id += 1;

        broker.send(b"", 0).unwrap();
    }
}
