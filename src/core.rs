/*
 * Eventlog Daemon
 * Joshua Bassett, 2017
 */

extern crate time;

use std::ops::Range;
use time::Timespec;

/**
 * Represents an entry in the eventlog.
 */
#[derive(Debug)]
pub struct Entry {
	pub id: i64,
	pub timestamp: Timespec,
	pub data: String
}

#[derive(Debug)]
pub enum Command {
    WriteEntry(Entry),
    DeleteRange(Range<i64>)
}

#[derive(Debug)]
pub enum Request {
    WriteData(String)
}
