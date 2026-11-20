//! Validate: a Stream held to its format's well-formedness, and to a
//! contract where the step names one, through the contract technology the
//! step's `format` names (ADR-0010, ADR-0042). The technologies are the
//! contract capability's own, linked here; nothing is parsed twice.
//! Whether a step validates at all stays the artifact's `validate`
//! (ADR-0031, amendment 2026-10-05). A custom contract is a unit of its
//! own module, with the capability `validate`, opened the same way.
//!
//! A contract reads nothing over a network: a reference outside a schema
//! is refused by the technology when the step is opened, never fetched.

use contract::{Contract, ContractFactory};
use node::Stage;
use node::unit::Unit;
use xcore::settings::{Applies, Given};

use crate::Values;

use contract::validate::{DECLARATION, VALIDATE as NAME};

/// Every contract technology linked into the standard module, by the word
/// a step's `format` names it with.
const FACTORIES: &[&dyn ContractFactory] = &[
    &asyncapi::AsyncApiFactory,
    &avro::AvroFactory,
    &csv::CsvFactory,
    &edifact::EdifactFactory,
    &x12::X12Factory,
    &fhir::FhirFactory,
    &fixed_width::FixedWidthFactory,
    &graphql_schema::GraphqlSchemaFactory,
    &hl7_v2::Hl7v2Factory,
    &json_schema::JsonSchemaFactory,
    &openapi::OpenApiFactory,
    &protobuf::ProtobufFactory,
    &regex::RegexContractFactory,
    &schematron::SchematronFactory,
    &sql::SqlFactory,
    &toml::TomlFactory,
    &toon::ToonFactory,
    &wsdl::WsdlFactory,
    &xml_schema::XmlSchemaFactory,
    &yaml::YamlFactory,
];

/// Validate, as the standard module declares it.
pub const UNIT: Unit<dyn Contract> = Unit::new(DECLARATION, open);

fn open(_: Stage, values: &Values) -> Result<Box<dyn Contract>, String> {
    let format = values.text("format");
    let factory = FACTORIES
        .iter()
        .find(|factory| factory.technology() == format)
        .ok_or_else(|| {
            let linked: Vec<&str> = FACTORIES
                .iter()
                .map(|factory| factory.technology())
                .collect();
            format!(
                "{NAME}: no contract technology is called '{format}'; this node has {}",
                linked.join(", ")
            )
        })?;
    let given: Vec<(String, Given)> = values.optional_text("contract").map_or_default(|contract| {
        vec![("reference".to_string(), Given::Text(contract.to_string()))]
    });
    factory
        .open(Applies::Both, &given)
        .map_err(|refused| format!("{NAME}: {refused}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use stream::Stream;

    #[test]
    fn a_format_no_technology_here_has_is_refused_naming_those_it_has() {
        let refused = opened(&[("format", "relax-ng")]).err().expect("refused");
        assert!(
            refused.contains("no contract technology is called 'relax-ng'"),
            "{refused}"
        );
        assert!(refused.contains("xml-schema"), "{refused}");
    }

    fn opened(given: &[(&str, &str)]) -> Result<Box<dyn Contract>, String> {
        let given: Vec<(String, Given)> = given
            .iter()
            .map(|(name, value)| ((*name).to_string(), Given::Text((*value).to_string())))
            .collect();
        UNIT.open(Stage::Receive, &given)
    }

    #[test]
    fn a_format_alone_holds_well_formedness() {
        let xml = opened(&[("format", "xml-schema")]).expect("opens");
        let stream =
            |text: &str| Stream::new(xcore::StreamId::new(1), text.as_bytes().to_vec(), None);
        assert!(xml.validate(&stream("<a><b/></a>")).expect("judged").valid);
        assert!(!xml.validate(&stream("<a><b></a>")).expect("judged").valid);
        let json = opened(&[("format", "json-schema")]).expect("opens");
        assert!(
            json.validate(&stream("{\"a\":[1,2]}"))
                .expect("judged")
                .valid
        );
        assert!(!json.validate(&stream("{\"a\":")).expect("judged").valid);
    }

    #[test]
    fn a_remote_reference_is_refused_when_the_step_opens_and_never_fetched() {
        let directory = file::temporary::Folder::new("unit-validate");
        let schema = directory.join("remote.json");
        std::fs::write(
            &schema,
            r#"{"properties":{"a":{"$ref":"https://example.test/a.json"}}}"#,
        )
        .expect("written");
        let place = schema.to_string_lossy().into_owned();
        let refused = opened(&[("format", "json-schema"), ("contract", &place)])
            .err()
            .expect("refused");
        assert!(refused.contains("over a network"), "{refused}");
        let xsd = directory.join("import.xsd");
        std::fs::write(
            &xsd,
            r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"><xs:import schemaLocation="https://example.test/b.xsd"/></xs:schema>"#,
        )
        .expect("written");
        let place = xsd.to_string_lossy().into_owned();
        let refused = opened(&[("format", "xml-schema"), ("contract", &place)])
            .err()
            .expect("refused");
        assert!(refused.contains("import"), "{refused}");
    }
}
