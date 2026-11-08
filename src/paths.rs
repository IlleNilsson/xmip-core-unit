//! Promote and Demote: a value read out of the content into the context
//! at receive, and written from the context into the content at send,
//! through the path language a step names — `XPath` for XML, `JSONPath` and the
//! content selector for JSON, an index for a CSV column, a segment and
//! element for EDI, a field for HL7 v2, a regular expression for text —
//! or, for Promote, a value the context already holds, a header the
//! transport set. Every path is compiled once, when its step is opened,
//! through the one path engine this module links its languages into, its
//! prefixes bound by the step's `namespaces` table and never by the content
//! (ADR-0066, amendment 2026-11-08).

use std::sync::LazyLock;

use demote::{Demotion, DemotionTarget, PayloadDemotion};
use node::Stage;
use node::unit::Unit;
use path::{Namespaces, Path, PathEngine};
use promote::Promotion;

use crate::Values;

use demote::declaration::DEMOTE;
use promote::declaration::{CONTEXT, PROMOTE};

/// The path languages linked, by the word a step's `language` names.
pub const LANGUAGES: &[&str] = &[
    "xpath",
    "jsonpath",
    "dot",
    "json-pointer",
    "index",
    "edi",
    "hl7-v2",
    "regex",
    "fhirpath",
];

/// The one engine, built the first time a step compiles a path.
fn engine() -> &'static PathEngine {
    static ENGINE: LazyLock<PathEngine> = LazyLock::new(|| {
        PathEngine::new(vec![
            Box::new(xpath::XPathLanguage),
            Box::new(jsonpath::JsonPathLanguage),
            Box::new(dot::DotLanguage),
            Box::new(json_pointer::JsonPointerLanguage),
            Box::new(index::IndexLanguage::default()),
            Box::new(edi::EdiLanguage),
            Box::new(hl7_v2_path::Hl7v2Language),
            Box::new(regex_path::RegexLanguage),
            Box::new(fhirpath::FhirPathLanguage),
        ])
    });
    &ENGINE
}

/// Promote, as the standard module declares it: at receive alone.
pub const PROMOTE_UNIT: Unit<Promotion> = Unit::new(promote::DECLARATION, open_promotion);

/// Demote, as the standard module declares it: at send alone.
pub const DEMOTE_UNIT: Unit<PayloadDemotion> = Unit::new(demote::DECLARATION, open_demotion);

/// The Path a step's `language`, `expression` and `namespaces` give
/// (ADR-0066, amendment 2026-11-08).
fn configured(values: &Values) -> Path {
    let namespaces: Namespaces = values
        .optional_table("namespaces")
        .unwrap_or_default()
        .iter()
        .map(|(prefix, uri)| (prefix.as_str(), uri.as_str()))
        .collect();
    Path::new(values.text("language"), values.text("expression")).with_namespaces(namespaces)
}

fn compiled(unit: &str, path: &Path) -> Result<path::CompiledPath, String> {
    let language = path.language.as_str();
    if !LANGUAGES.contains(&language) {
        return Err(format!(
            "{unit}: no path language is called '{language}'; this node has {}",
            LANGUAGES.join(", ")
        ));
    }
    engine()
        .compile(path)
        .map_err(|refused| format!("{unit}: {refused}"))
}

fn open_promotion(_: Stage, values: &Values) -> Result<Box<Promotion>, String> {
    let (property, path) = (values.text("property"), configured(values));
    Ok(Box::new(if path.language == CONTEXT {
        path.namespaces
            .unused_by(CONTEXT)
            .map_err(|refused| format!("{PROMOTE}: {refused}"))?;
        Promotion::context(property, path.expression)
    } else {
        Promotion::content(property, compiled(PROMOTE, &path)?)
    }))
}

fn open_demotion(_: Stage, values: &Values) -> Result<Box<PayloadDemotion>, String> {
    let target_path = configured(values);
    compiled(DEMOTE, &target_path)?;
    let demotion = Demotion {
        context_key: values.text("property").to_string(),
        target: DemotionTarget::PayloadElement,
        target_path,
    };
    demotion
        .compile(engine())
        .map_err(|refused| format!("{DEMOTE}: {refused}"))?
        .map(Box::new)
        .ok_or_else(|| format!("{DEMOTE}: the value is written into the content alone"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::MessageContext;
    use path::{Content, Rewriting};
    use stream::Stream;
    use xcore::settings::Given;
    use xcore::{ScalarValue, StreamId};

    fn given(settings: &[(&str, &str)]) -> Vec<(String, Given)> {
        settings
            .iter()
            .map(|(name, value)| ((*name).to_string(), Given::Text((*value).to_string())))
            .collect()
    }

    fn promoted(language: &str, expression: &str, content: &str) -> ScalarValue {
        let unit = PROMOTE_UNIT
            .open(
                Stage::Receive,
                &given(&[
                    ("property", "Value"),
                    ("language", language),
                    ("expression", expression),
                ]),
            )
            .expect("opens");
        let stream = Stream::new(StreamId::new(1), content.as_bytes().to_vec(), None);
        unit.promote(&Content::of(&stream), MessageContext::new())
            .expect("promotes")
            .get("Value")
            .cloned()
            .expect("a value promoted")
    }

    fn text(value: &str) -> ScalarValue {
        ScalarValue::Text(value.into())
    }

    #[test]
    fn every_language_promotes_from_its_content() {
        assert_eq!(
            promoted("xpath", "/Order/Id", "<Order><Id>A-1</Id></Order>"),
            text("A-1")
        );
        assert_eq!(
            promoted("jsonpath", "$.order.id", r#"{"order":{"id":"A-1"}}"#),
            text("A-1")
        );
        assert_eq!(
            promoted("dot", "order.id", r#"{"order":{"id":"A-1"}}"#),
            text("A-1")
        );
        assert_eq!(
            promoted("index", "2/2", "id,name\nA-1,Jane\n"),
            text("Jane")
        );
        assert_eq!(
            promoted(
                "edi",
                "BGM/2",
                "UNH+1+ORDERS:D:96A:UN'BGM+220+PO4711'UNT+3+1'"
            ),
            text("PO4711")
        );
        assert_eq!(
            promoted(
                "hl7-v2",
                "PID-5-2",
                "MSH|^~\\&|A|B|C|D|20260101||ADT^A01|1|P|2.5\rPID|1||9||DOE^JOHN\r"
            ),
            text("JOHN")
        );
    }

    #[test]
    fn a_step_binds_its_prefixes_through_its_namespaces_and_refuses_one_it_does_not_bind() {
        let mut settings = given(&[
            ("property", "Value"),
            ("language", "xpath"),
            ("expression", "/o:Order/o:Id"),
        ]);
        let unbound = PROMOTE_UNIT
            .open(Stage::Receive, &settings)
            .expect_err("refused");
        assert!(
            unbound.contains("uses the prefix o, which the path's namespaces do not bind"),
            "{unbound}"
        );
        settings.push((
            "namespaces".to_string(),
            Given::Table(vec![(
                "o".to_string(),
                Given::Text("urn:example:order".into()),
            )]),
        ));
        let unit = PROMOTE_UNIT.open(Stage::Receive, &settings).expect("opens");
        let stream = Stream::new(
            StreamId::new(1),
            br#"<ord:Order xmlns:ord="urn:example:order"><ord:Id>A-1</ord:Id></ord:Order>"#
                .to_vec(),
            None,
        );
        let promoted = unit
            .promote(&Content::of(&stream), MessageContext::new())
            .expect("promotes");
        assert_eq!(promoted.get("Value"), Some(&text("A-1")));

        settings[1].1 = Given::Text(CONTEXT.into());
        let refused = PROMOTE_UNIT
            .open(Stage::Receive, &settings)
            .expect_err("refused");
        assert!(
            refused.contains("context has no namespaces, and the path binds o"),
            "{refused}"
        );
    }

    #[test]
    fn a_promotion_from_the_context_reads_a_header_the_transport_set() {
        let unit = PROMOTE_UNIT
            .open(
                Stage::Receive,
                &given(&[
                    ("property", "Channel"),
                    ("language", CONTEXT),
                    ("expression", "http.x-channel"),
                ]),
            )
            .expect("opens");
        let stream = Stream::new(StreamId::new(1), Vec::new(), None);
        let context =
            MessageContext::new().with_value("http.x-channel", ScalarValue::Text("web".into()));
        let promoted = unit
            .promote(&Content::of(&stream), context)
            .expect("promotes");
        assert_eq!(promoted.get("Channel"), Some(&text("web")));
    }

    #[test]
    fn a_demotion_writes_the_value_back_into_the_content() {
        let unit = DEMOTE_UNIT
            .open(
                Stage::Send,
                &given(&[
                    ("property", "Status"),
                    ("language", "xpath"),
                    ("expression", "/Order/Status"),
                ]),
            )
            .expect("opens");
        let source = Stream::new(
            StreamId::new(1),
            b"<Order><Status>open</Status></Order>".to_vec(),
            None,
        );
        let mut rewriting = Rewriting::of(&source, StreamId::new(2));
        let context =
            MessageContext::new().with_value("Status", ScalarValue::Text("closed".into()));
        demote::apply_to_structure(&context, &mut rewriting, std::slice::from_ref(&*unit))
            .expect("writes");
        let written = rewriting.finish().expect("finishes");
        assert!(
            String::from_utf8_lossy(written.load().expect("held"))
                .contains("<Status>closed</Status>")
        );
    }

    #[test]
    fn a_language_no_node_has_and_a_stage_of_the_other_side_are_refused() {
        let refused = PROMOTE_UNIT
            .open(
                Stage::Receive,
                &given(&[
                    ("property", "A"),
                    ("language", "xquery"),
                    ("expression", "x"),
                ]),
            )
            .expect_err("refused");
        assert!(
            refused.contains("no path language is called 'xquery'"),
            "{refused}"
        );
        let refused = DEMOTE_UNIT
            .open(
                Stage::Receive,
                &given(&[
                    ("property", "A"),
                    ("language", "xpath"),
                    ("expression", "/a"),
                ]),
            )
            .expect_err("refused");
        assert!(refused.contains("does not run at receive"), "{refused}");
    }
}
