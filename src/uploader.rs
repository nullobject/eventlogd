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
use std::sync::mpsc::Receiver;
use std::thread;

use command::Command::{self, WriteEntry};
use entry::Entry;

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

    // Write a payload to the buffer.
    fn write(&mut self, entry: Entry) -> Result<(), Error> {
        debug!("Writing {:?} to buffer", entry);
        if self.queue.len() >= GATEWAY_BUFFER_SIZE {
            self.flush().unwrap();
        }
        self.queue.push(entry);
        Ok(())
    }

    // Flush the buffer to S3.
    fn flush(&mut self) -> Result<(), Error> {
        debug!("Flushing buffer to S3");
        {
            let first = self.queue.first().unwrap();
            let last = self.queue.last().unwrap();
            // TODO: Is this the most idomatic way to do this?
            let payload = self.queue
                .iter()
                .fold(String::new(), |acc, x| acc + &x.payload + "\n");
            let compressed = compress(payload.as_bytes(), EXTREME_PRESET).unwrap();
            let filename = format!("{}-{}.xz", first.id, last.id);
            let req = PutObjectRequest {
                bucket: "test.joshbassett.info".to_owned(),
                key: filename,
                body: Some(compressed),
                ..Default::default()
            };
            self.s3.put_object(&req).unwrap();
        }
        self.queue.clear();
        debug!("Done!");
        Ok(())
    }
}

// Read payloads from queue and upload them.
fn uploader(rx: Receiver<Command>) -> Result<(), Error> {
    let mut gateway = Gateway::new();

    loop {
        let command = rx.recv().unwrap();

        match command {
            WriteEntry(entry) => {
                gateway.write(entry).unwrap();
            }
        }
    }
}

pub fn spawn_uploader(rx: Receiver<Command>) -> Result<(), Error> {
    thread::spawn(|| {
        uploader(rx).unwrap();
    });

    Ok(())
}
