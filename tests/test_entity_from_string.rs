use triple_s::{
    Character,
    Code,
    Format,
    Language,
    Mode,
    Multiple,
    Position,
    Quantity,
    RangeDate,
    RangeTime,
    Record,
    Response,
    Single,
    Spread,
    Survey,
    Text,
    TextVariant,
    Version,
};

const CLASSIC_SINGLE: &str = r#"
VARIABLE 1
NAME "AGE"
LABEL "Age"
POSITION 1
TYPE SINGLE
VALUES
1 "Under 18"
END VALUES
END VARIABLE
"#;

const XML_MULTIPLE: &str = r#"
<variable ident="2" type="multiple">
    <name>FEATURES</name>
    <label>Features<text xml:lang="en-GB" mode="interview">Which features?</text></label>
    <position start="2" finish="4"/>
    <spread subfields="3"/>
    <values><value code="1">Online ordering</value></values>
</variable>
"#;

#[test]
fn classic_fragments_use_the_existing_parser() {
    let question = triple_s::Question::from_string(CLASSIC_SINGLE).unwrap();
    assert_eq!(question.name, "AGE");
    assert_eq!(
        Record::from_string(format!("RECORD A\n{CLASSIC_SINGLE}\nEND RECORD"))
            .unwrap()
            .id,
        'A'
    );
    assert_eq!(Text::from_string(CLASSIC_SINGLE).unwrap().value, "Age");
    assert_eq!(Text::from_string(r#"LABEL "Age""#).unwrap().value, "Age");
    assert_eq!(Position::from_string(CLASSIC_SINGLE).unwrap().from, 1);
    assert!(matches!(
        Response::from_string(CLASSIC_SINGLE).unwrap(),
        Response::Single(_)
    ));
    assert!(matches!(
        Format::from_string(CLASSIC_SINGLE).unwrap(),
        Format::Numeric
    ));
    assert_eq!(
        Single::<u64>::from_string(CLASSIC_SINGLE).unwrap().values.len(),
        1
    );
    assert_eq!(Code::<u64>::from_string(CLASSIC_SINGLE).unwrap().code, 1);
    assert_eq!(Version::from_string("VERSION 1.1").unwrap(), Version::V1_1);
    assert_eq!(
        Quantity::from_string("TYPE QUANTITY\nVALUES\n0 TO 10\nEND VALUES")
            .unwrap()
            .range
            .end,
        10.0
    );
}

#[test]
fn xml_fragments_preserve_nested_entities() {
    let question = triple_s::Question::from_string(XML_MULTIPLE).unwrap();
    assert_eq!(question.name, "FEATURES");
    assert_eq!(
        Multiple::from_string(XML_MULTIPLE).unwrap().spread.unwrap().subfield,
        3
    );
    assert_eq!(Spread::from_string(XML_MULTIPLE).unwrap().subfield, 3);
    assert_eq!(
        Code::<u64>::from_string(XML_MULTIPLE).unwrap().label.value,
        "Online ordering"
    );

    let text = Text::from_string(XML_MULTIPLE).unwrap();
    assert_eq!(text.variant.len(), 1);
    assert_eq!(
        TextVariant::from_string(XML_MULTIPLE).unwrap().value,
        "Which features?"
    );
    assert_eq!(
        TextVariant::from_string(
            r#"<text xml:lang="en-GB" mode="interview">Which features?</text>"#
        )
        .unwrap()
        .mode,
        Mode::Interview
    );
    assert_eq!(Language::from_string("en-GB").unwrap().to_string(), "en-GB");
    assert_eq!(Mode::from_string("interview").unwrap(), Mode::Interview);
}

#[test]
fn response_specific_fragments_extract_the_requested_type() {
    let quantity = r#"<variable ident="1" type="quantity"><name>Q</name><label>Q</label><position start="1"/><values><range from="0" to="10"/><value code="99">Missing</value></values></variable>"#;
    assert_eq!(Quantity::from_string(quantity).unwrap().range.end, 10.0);
    assert_eq!(Code::<f64>::from_string(quantity).unwrap().code, 99.0);

    let character = r#"<variable ident="1" type="character"><name>C</name><label>C</label><position start="1"/><size>12</size></variable>"#;
    assert_eq!(Character::from_string(character).unwrap().size, 12);

    let literal = r#"<variable ident="1" type="single" format="literal"><name>L</name><label>L</label><position start="1"/><values><value code="A">Alpha</value></values></variable>"#;
    assert_eq!(
        Single::<String>::from_string(literal).unwrap().values.len(),
        1
    );
    assert_eq!(Code::<String>::from_string(literal).unwrap().code, "A");

    let date = r#"<variable ident="1" type="date"><name>D</name><label>D</label><position start="1"/><values><range from="20240101" to="20241231"/></values></variable>"#;
    assert!(RangeDate::from_string(date).unwrap().range.is_some());

    let time = r#"<variable ident="1" type="time"><name>T</name><label>T</label><position start="1"/><values><range from="000000" to="235959"/></values></variable>"#;
    assert!(RangeTime::from_string(time).unwrap().range.is_some());

    let document = r#"<sss version="1.2"><survey><record ident="A">{XML_MULTIPLE}</record></survey></sss>"#;
    let document = document.replace("{XML_MULTIPLE}", XML_MULTIPLE);
    assert_eq!(
        Survey::from_string(document).unwrap().record.content.len(),
        1
    );
}
