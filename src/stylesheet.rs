//! Stylesheet: `xmip-core-transform`'s XSLT unit over the engines this
//! module links, none yet: an engine is a technology of the trait, linked
//! here or loaded as a module of its own (ADR-0061), and admitted only where
//! it reaches no network (open problem 53); a step chooses by `engine`, and
//! one naming an engine not linked is refused, in words, when it is opened.

use node::Stage;
use node::unit::Unit;
use transform::Transform;
use transform::stylesheet::{DECLARATION, StylesheetEngine};

use crate::Values;

/// The engines linked.
const ENGINES: &[&dyn StylesheetEngine] = &[];

/// Stylesheet, as the standard module declares it.
pub const UNIT: Unit<dyn Transform> = Unit::new(DECLARATION, open);

fn open(stage: Stage, values: &Values) -> Result<Box<dyn Transform>, String> {
    transform::stylesheet::open(ENGINES, stage, values)
}
