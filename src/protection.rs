//! Encryption and Signature: `xmip-core-prepare`'s units over the engines
//! this module links, the first `rpgp` for `OpenPGP`. CMS and JOSE are
//! formats a step may name and no engine here implements yet: a step
//! naming one is refused, in words, when it is opened.

use node::Stage;
use node::unit::Unit;
use prepare::Prepare;
use prepare::declaration::{ENCRYPTION_UNIT, SIGNATURE_UNIT};
use prepare::protection::ProtectionEngine;

use crate::Values;

/// The engines linked.
const ENGINES: &[&dyn ProtectionEngine] = &[&rpgp::Rpgp];

/// Encryption, as the standard module declares it.
pub const ENCRYPTION: Unit<dyn Prepare> = Unit::new(ENCRYPTION_UNIT, encryption);

/// Signature, as the standard module declares it.
pub const SIGNATURE: Unit<dyn Prepare> = Unit::new(SIGNATURE_UNIT, signature);

fn encryption(stage: Stage, values: &Values) -> Result<Box<dyn Prepare>, String> {
    prepare::protection::open_encryption(ENGINES, stage, values)
}

fn signature(stage: Stage, values: &Values) -> Result<Box<dyn Prepare>, String> {
    prepare::protection::open_signature(ENGINES, stage, values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xcore::settings::Given;

    #[test]
    fn a_format_no_engine_here_implements_is_refused_in_words() {
        let given = |format: &str| {
            vec![
                ("format".to_string(), Given::Text(format.to_string())),
                ("engine".to_string(), Given::Text("any".to_string())),
                ("key".to_string(), Given::Text("key.pem".to_string())),
            ]
        };
        let refused = ENCRYPTION
            .open(Stage::Receive, &given("cms"))
            .err()
            .expect("refused");
        assert!(refused.contains("no cms engine called 'any'"), "{refused}");
        let refused = SIGNATURE
            .open(Stage::Send, &given("openpgp"))
            .err()
            .expect("refused");
        assert!(refused.contains("it has rpgp"), "{refused}");
    }
}
