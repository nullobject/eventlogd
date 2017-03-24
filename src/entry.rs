/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 */

extern crate time;

/*
 * Represents an entry in the eventlog.
 */
#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Entry {
	pub id: u32,
	pub timestamp: String,
	pub payload: String
}

impl Entry {
    pub fn new(id: u32, payload: String) -> Self {
        let timestamp = time::strftime("%Y-%m-%d %H:%M:%S.%f", &time::now()).unwrap();
        Entry {
            id: id,
            timestamp: timestamp,
            payload: payload.to_owned()
        }
    }
}
