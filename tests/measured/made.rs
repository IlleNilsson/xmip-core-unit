//! A JSON array of small objects, made as it is read and kept as a Stream:
//! what the bounded-memory test (`tests/bounded.rs`) and the module's
//! timing program both validate. Shared, not copied.

use std::io::{self, Read};
use std::sync::Arc;

use stream::{Content, Stream};
use xcore::StreamId;

/// A JSON array of `count` small objects, made as it is read.
struct Made {
    count: u64,
}

struct Reading {
    count: u64,
    at: u64,
    pending: Vec<u8>,
}

impl Content for Made {
    fn reader(&self) -> io::Result<Box<dyn Read + Send + '_>> {
        Ok(Box::new(Reading {
            count: self.count,
            at: 0,
            pending: b"[".to_vec(),
        }))
    }
}

impl Read for Reading {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        while self.pending.is_empty() && self.at <= self.count {
            self.pending = if self.at < self.count {
                let comma = if self.at > 0 { "," } else { "" };
                format!(
                    "{comma}{{\"id\":{},\"name\":\"Jane & Co\",\"total\":12.5}}",
                    self.at
                )
                .into_bytes()
            } else {
                b"]".to_vec()
            };
            self.at += 1;
        }
        let taken = out.len().min(self.pending.len());
        out[..taken].copy_from_slice(&self.pending[..taken]);
        self.pending.drain(..taken);
        Ok(taken)
    }
}

pub fn kept(count: u64) -> Stream {
    let made = Made { count };
    let length = io::copy(&mut made.reader().expect("a reader"), &mut io::sink()).expect("made");
    Stream::kept(StreamId::new(1), length, None, Arc::new(made))
}
