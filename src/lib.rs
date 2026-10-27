#![forbid(unsafe_code)]

//! The standard units, linked: Xmip's own module of compiled units, out of
//! the box at receive, at process and at send (the owner, 2026-10-22:
//! *standard 'functions' like Xml, Json and what not should be there out of
//! the box for receive, process and send*; *Validation according to format
//! or custom contracts has to be in there*). A step names one as
//! `module = "xmip"`, the module built in: no `[[modules]]` entry declares
//! it, and the configuration knows its manifest itself
//! (`configure::standard`), written from the declarations each capability
//! gives (ADR-0066, amendment 2026-10-22).
//!
//! Each unit is implemented once, in the capability it belongs to, and
//! declared there beside its code; this crate links the code and the
//! technologies, hands each unit the technologies it chooses among — the
//! contract technologies to Validate, the path languages to Promote and
//! Demote, `xrust` to Stylesheet, `rpgp` to Encryption and Signature — and
//! opens them.
//!
//! Every standard unit works offline (ADR-0045, amendment 2026-10-22): no
//! unit, technology or dependency here holds code that reaches a network
//! (`test/Offline.Test.ps1` in the estate).

pub mod paths;
pub mod protection;
pub mod stylesheet;
pub mod validate;

/// What a step gave a unit, read through its declaration.
pub type Values = xcore::settings::Read;

/// The module's name, as a step entry's `module` names it.
pub const MODULE: &str = "xmip";

/// Every unit this module opens, by its declaration, in the order a person
/// reads them.
#[must_use]
pub fn declarations() -> Vec<&'static node::unit::Declaration> {
    let mut every: Vec<&'static node::unit::Declaration> =
        prepare::UNITS.iter().map(|unit| unit.declaration).collect();
    every.push(protection::ENCRYPTION.declaration);
    every.push(protection::SIGNATURE.declaration);
    every.push(validate::UNIT.declaration);
    every.push(paths::PROMOTE_UNIT.declaration);
    every.push(paths::DEMOTE_UNIT.declaration);
    every.extend(transform::UNITS.iter().map(|unit| unit.declaration));
    every.push(stylesheet::UNIT.declaration);
    every
}

/// The units declared and not built, which this module opens none of:
/// Debatch, which an Xmip Work Process's engine will call (open problem
/// 47).
pub const UNBUILT: [&node::unit::Declaration; 1] = [process::declaration::DEBATCH];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_standard_unit_is_opened_here_and_no_other() {
        let given: Vec<&node::unit::Declaration> = prepare::DECLARATIONS
            .into_iter()
            .chain([
                contract::validate::DECLARATION,
                promote::DECLARATION,
                demote::DECLARATION,
            ])
            .chain(transform::DECLARATIONS)
            .collect();
        let opened = declarations();
        assert!(
            UNBUILT
                .iter()
                .all(|unbuilt| process::DECLARATIONS.contains(unbuilt))
        );
        for declaration in &given {
            assert!(
                opened.iter().any(|unit| *unit == *declaration),
                "{} is declared and not opened here",
                declaration.name
            );
        }
        assert_eq!(
            opened.len(),
            given.len(),
            "a unit is opened twice or undeclared"
        );
    }
}
