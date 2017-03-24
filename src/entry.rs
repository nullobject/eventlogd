/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 */

extern crate time;

use time::Timespec;

/*
 * Represents an entry in the eventlog.
 */
#[derive(Debug)]
pub struct Entry {
	pub id: i64,
	pub timestamp: Timespec,
	pub data: String
}
