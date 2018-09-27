#[macro_use]
extern crate slog ;
extern crate slog_async;
extern crate slog_term;
#[macro_use]
extern crate lazy_static;

use std::io::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use std::process::id;
use std::result;
use std::io;
use std::thread::sleep;
use std::hash::Hash;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::fs::OpenOptions;
use slog::Drain;
use slog::*;
use slog::Key;

lazy_static! {
    pub static ref logger: slog::Logger = build_logger();
}

fn build_logger() -> slog::Logger {
    let log_path = "target/your_log_file_path.log";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let mut plain = PlainDrain { file: RefCell::new(file) };
    let drain = slog_async::Async::default(plain).fuse();

    slog::Logger::root(drain, o!())
}

struct PlainDrain {
    file: RefCell<File>
}

impl Drain for PlainDrain
{
    type Err = Never;
    type Ok = ();

    fn log(&self, record: &Record, values: &OwnedKVList) -> result::Result<(), Never> {
        self.file.borrow_mut().write_fmt(*record.msg());
        self.file.borrow_mut().write(b"\n");
        Ok(())
    }
}


pub struct Tracepoint<'a> {
  location: &'a str
}

impl<'a> Tracepoint<'a> {
    pub fn new(location: &str) ->Tracepoint {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let us = since_the_epoch.as_secs() * 1_000_000 + since_the_epoch.subsec_nanos() as u64 / 1000;


        let mut hasher = DefaultHasher::new();
        let tid = std::thread::current().id();
        tid.hash(&mut hasher);
        let tidhash = hasher.finish();

        let h = 0;
        trace!(logger, r#"{{"name": "{}", "cat": "perf", "ph": "B", "ts": {}, "pid": {}, "tid": {:?}}},"#,
               location, us, std::process::id(), tidhash);

        return Tracepoint {
          location
        };
    }
}

impl<'a> Drop for Tracepoint<'a> {
    fn drop(&mut self) {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let us = since_the_epoch.as_secs() * 1_000_000 + since_the_epoch.subsec_nanos() as u64 / 1000;

        let mut hasher = DefaultHasher::new();
        let tid = std::thread::current().id();
        tid.hash(&mut hasher);
        let tidhash = hasher.finish();

        let h = 0;
        trace!(logger, r#"{{"name": "{}", "cat": "perf", "ph": "E", "ts": {}, "pid": {}, "tid": {:?}}},"#,
               self.location, us, std::process::id(), tidhash);
    }
}

#[macro_export]
macro_rules! TRACE {
    ($x:expr) => { let _t = Tracepoint::new($x); }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use Tracepoint;
        use std::time;
        use std::thread;
        use std::thread::sleep;
        extern crate env_logger;

        env_logger::init();

        for i in 0..10 {
            TRACE!("here");

            let ten_millis = time::Duration::from_millis(10);
            let now = time::Instant::now();

            thread::sleep(ten_millis);
        }
    }
}
