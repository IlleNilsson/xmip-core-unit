//! Validate holds a JSON Stream far larger than the pieces it reads to its
//! well-formedness in bounded memory. How fast it validates a small one is
//! the module's timing program's (`tests/measured`). The other formats'
//! technologies read a Stream whole for now, which open problem 50 records
//! with each one's bound.
//!
//! One test in a program of its own: the allocator's count is the whole
//! process's, so nothing runs beside it.

#[path = "measured/made.rs"]
mod made;

use std::time::Instant;

use made::kept;
use node::Stage;
use xcore::settings::Given;

#[global_allocator]
static HELD: xcore::held::Counted = xcore::held::Counted;

#[test]
fn validate_holds_a_large_json_stream_in_bounded_memory() {
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
}
