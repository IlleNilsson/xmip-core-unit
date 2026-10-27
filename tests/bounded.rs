//! Validate holds a JSON Stream far larger than the pieces it reads to its
//! well-formedness in bounded memory, and a small one within the
//! millisecond rule. The other formats' technologies read a Stream whole
//! for now, which open problem 50 records with each one's bound.
//!
//! One test in a program of its own: the allocator's count is the whole
//! process's, so nothing runs beside it.

use std::io::{self, Read};
use std::sync::Arc;
use std::time::{Duration, Instant};

use node::Stage;
use stream::{Content, Stream};
use xcore::StreamId;
use xcore::settings::Given;

#[global_allocator]
static HELD: xcore::held::Counted = xcore::held::Counted;

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

fn kept(count: u64) -> Stream {
    let made = Made { count };
    let length = io::copy(&mut made.reader().expect("a reader"), &mut io::sink()).expect("made");
    Stream::kept(StreamId::new(1), length, None, Arc::new(made))
}

#[test]
fn validate_holds_a_large_json_stream_in_bounded_memory_and_a_small_one_at_once() {
    let contract = xmip_core_unit::validate::UNIT
        .open(
            Stage::Receive,
            &[("format".to_string(), Given::Text("json-schema".to_string()))],
        )
        .expect("opens");
    let large = kept(1_200_000);
    let started = Instant::now();
    let (held, judged) = xcore::held::above(|| contract.validate(&large).expect("judged"));
    println!(
        "Validate json-schema, bare: {} MiB in {:?}, {} KiB held at most",
        large.length() / 1024 / 1024,
        started.elapsed(),
        held / 1024
    );
    assert!(judged.valid, "{:?}", judged.issues);
    assert!(held < 1024 * 1024, "held {held} bytes");
    let small = kept(10);
    let rounds: Vec<Duration> = (0..50)
        .map(|_| {
            let started = Instant::now();
            assert!(contract.validate(&small).expect("judged").valid);
            started.elapsed()
        })
        .collect();
    let spread = xcore::latency::spread("Validate json-schema, ten records", rounds);
    assert!(
        spread.fastest_of_five < Duration::from_millis(1),
        "{spread:?}"
    );
}
