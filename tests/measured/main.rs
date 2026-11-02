//! The module's timing program (`doc/governance/rust-style.md` section
//! 5c): Validate held to the millisecond rule. A plain `cargo test` leaves
//! it out; the landing runs it after every other test of the module, alone,
//! one test at a time (`cargo test --test measured --all-features --
//! --test-threads=1`), so what is measured is Xmip's work and not its
//! neighbors':
//!
//! - Validate judges a small JSON Stream well within a millisecond, made
//!   as `tests/bounded.rs` makes its large one (`made.rs`, shared).

mod made;

use std::time::{Duration, Instant};

use made::kept;
use node::Stage;
use xcore::settings::Given;

#[test]
fn validate_judges_a_small_json_stream_at_once() {
    let contract = xmip_core_unit::validate::UNIT
        .open(
            Stage::Receive,
            &[("format".to_string(), Given::Text("json-schema".to_string()))],
        )
        .expect("opens");
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
