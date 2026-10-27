//! Stylesheet: `xmip-core-transform`'s XSLT unit over the engines this
//! module links, the first `xrust` (Apache-2.0; XSLT 1.0). Another engine
//! — SaxonC-HE among them — is a technology of the same trait, linked here
//! or loaded as a module of its own (ADR-0061); a step chooses by `engine`.

use node::Stage;
use node::unit::Unit;
use transform::Transform;
use transform::stylesheet::{DECLARATION, StylesheetEngine};

use crate::Values;

/// The engines linked.
const ENGINES: &[&dyn StylesheetEngine] = &[&xrust_engine::Xrust];

/// Stylesheet, as the standard module declares it.
pub const UNIT: Unit<dyn Transform> = Unit::new(DECLARATION, open);

fn open(stage: Stage, values: &Values) -> Result<Box<dyn Transform>, String> {
    transform::stylesheet::open(ENGINES, stage, values)
}
