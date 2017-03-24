use entry::Entry;

#[derive(Debug)]
pub enum Command {
    WriteEntry(Entry)
}
