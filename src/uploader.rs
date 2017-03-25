/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 *
 * TODO: Load the journal into the queue when the server restarts.
 */

extern crate lzma;
extern crate rusoto;
extern crate hyper;

use self::hyper::Client;
use self::lzma::{compress, EXTREME_PRESET};
use self::rusoto::{DefaultCredentialsProvider, Region, default_tls_client};
use self::rusoto::s3::{PutObjectRequest, S3Client};
use std::io::Error;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

use core::Command::{self, WriteEntry};
use core::Entry;
use core::Response::{self, DeleteRange, Lol};

const GATEWAY_BUFFER_SIZE: usize = 5;

struct Gateway {
    s3: S3Client<DefaultCredentialsProvider, Client>,
    queue: Vec<Entry>
}

impl Gateway {
    fn new() -> Self {
        let credentials_provider = DefaultCredentialsProvider::new().unwrap();
        Gateway {
            s3: S3Client::new(default_tls_client().unwrap(), credentials_provider, Region::UsEast1),
            queue: Vec::with_capacity(GATEWAY_BUFFER_SIZE)
        }
    }

    // Write a data to the buffer.
    fn write(&mut self, entry: Entry) -> Result<Response, Error> {
        let mut response = Lol;

        if self.queue.len() >= GATEWAY_BUFFER_SIZE {
            response = self.flush().unwrap();
        }

        debug!("Writing {:?} to buffer", entry);

        // Enqueue the entry.
        self.queue.push(entry);

        Ok(response)
    }

    // Flush the buffer to S3.
    fn flush(&mut self) -> Result<Response, Error> {
        let range;

        debug!("Flushing buffer to S3");

        {
            let first = self.queue.first().unwrap();
            let last = self.queue.last().unwrap();
            // TODO: Is this the most idomatic way to do this?
            let data = self.queue
                .iter()
                .fold(String::new(), |acc, x| acc + &x.data + "\n");
            let compressed = compress(data.as_bytes(), EXTREME_PRESET).unwrap();
            range = first.id..last.id;
            let filename = format!("{}-{}.xz", first.id, last.id);
            let req = PutObjectRequest {
                bucket: "test.joshbassett.info".to_owned(),
                key: filename,
                body: Some(compressed),
                ..Default::default()
            };

            self.s3.put_object(&req).unwrap();
        }

        // Clear the queue.
        self.queue.clear();

        debug!("Done!");

        Ok(DeleteRange(range))
    }
}

// Read data from queue and upload them.
fn uploader(rx: Receiver<Command>, tx: Sender<Response>) -> Result<(), Error> {
    let mut gateway = Gateway::new();

    loop {
        let command = rx.recv().unwrap();

        match command {
            WriteEntry(entry) => {
                let response = gateway.write(entry).unwrap();
                tx.send(response).unwrap()
            }
        }
    }
}

pub fn spawn_uploader(rx: Receiver<Command>, tx: Sender<Response>) -> Result<(), Error> {
    thread::spawn(|| {
        uploader(rx, tx).unwrap();
    });

    Ok(())
}
