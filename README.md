# xmip-core-unit

The standard units: Xmip's own module of compiled units, out of the box at
receive, at process and at send (the owner, 2026-10-22: *standard
'functions' like Xml, Json and what not should be there out of the box for
receive, process and send*; *Validation according to format or custom
contracts has to be in there*). They are the module built in, `xmip`: a
step entry names a standard unit as it names anyone's, `module = "xmip"`,
`unit = "Compression"`, and no `[[modules]]` entry declares the module
(ADR-0066, amendment 2026-10-22).

Each unit is implemented once, in the capability it belongs to, and declared
there beside its code as data (`xmip-core-node`'s `unit::Declaration`);
`xmip-core-configure`'s `standard` writes the module's manifest from those
declarations without linking a unit. This crate links the code, hands each
unit the technologies it chooses among — the contract technologies to
Validate, the path languages to Promote and Demote, `rpgp` to Encryption
and Signature, no engine yet to Stylesheet (open problem 53) — and opens
them; a test holds that every declared unit is opened here and nothing else.

| Capability | Units |
| --- | --- |
| prepare | Compression, CharacterEncoding, TransferEncoding, TextNormalization, Encryption, Signature (`xmip-core-prepare`) |
| validate | Validate: `format`, a contract technology, and `contract`, its reference |
| promote | Promote, at receive: `property`, `language`, `expression` |
| demote | Demote, at send: `property`, `language`, `expression` |
| transform | Convert, Stylesheet, Script (`xmip-core-transform`) |
| work-process | Debatch — declared, not built |

**Offline** (ADR-0045, amendment 2026-10-22): the manifest says
`offline = true` of the module and of every unit, and `test/Offline.Test.ps1`
in the estate reads every package this crate's build compiles for code that
reaches a network.

Not built: running a unit — a node refuses a Port or Location that attaches a
step until the runtime runs them; CMS and JOSE, which no linked engine
implements; Debatch, which the Work Process engine will call.

## Toolchain

`rust-toolchain.toml` pins the toolchain for the whole estate. Do not change it
here.

## Verification

The included workflow is manual-only and calls the versioned shared workflow at
`IlleNilsson/.github@v1`.
