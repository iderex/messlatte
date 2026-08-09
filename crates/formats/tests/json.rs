//! The header container: what it reads, what it refuses, and the order it
//! writes in (#35).

use messlatte_formats::json::{self, Json, Object};

#[test]
fn a_document_is_written_in_key_order_whatever_order_it_was_built_in() {
    // The property the case is about. Two writers that agree on the content and
    // disagree on the order produce two documents with two hashes, and the case
    // index in #39 would then record a change in physics that nobody made.
    let mut built = Object::new();
    built.insert("zulu".to_string(), Json::Number(1.0));
    built.insert("alpha".to_string(), Json::Number(2.0));
    built.insert("Mike".to_string(), Json::Bool(true));
    assert_eq!(
        text(&Json::Object(built)),
        "{\"Mike\":true,\"alpha\":2,\"zulu\":1}\n"
    );
}

#[test]
fn a_canonical_document_read_and_written_again_is_the_same_bytes() {
    let original = "{\"a\":[1,-2.5,0.0000003,0],\"b\":{\"c\":null},\"d\":\"x\\ny\",\"e\":false}\n";
    let parsed = json::parse(original.as_bytes()).expect("the document is readable");
    assert_eq!(text(&parsed), original);
}

#[test]
fn a_document_written_elsewhere_is_normalised_rather_than_echoed() {
    // The canonical form is what this writer produces and not what the input
    // happened to look like. A number has several spellings and one canonical
    // one, so a header written by somebody else hashes as what it means rather
    // than as how it was typed. The value has to survive that, which is the
    // second half of the case.
    let foreign = b"{ \"a\" : [ 3e-7, 1E2, 0.5 ] }";
    let parsed = json::parse(foreign).expect("readable");
    assert_eq!(text(&parsed), "{\"a\":[0.0000003,100,0.5]}\n");

    let again = json::parse(text(&parsed).as_bytes()).expect("readable");
    let bits: Vec<Option<u64>> = again
        .get("a")
        .and_then(Json::as_array)
        .expect("an array")
        .iter()
        .map(|item| item.as_number().map(f64::to_bits))
        .collect();
    assert_eq!(
        bits,
        vec![
            Some(3e-7f64.to_bits()),
            Some(1e2f64.to_bits()),
            Some(0.5f64.to_bits())
        ]
    );
}

#[test]
fn a_double_survives_the_document_bit_for_bit() {
    // The header carries the axes, so a value that came back as its own
    // neighbour would move a sample. Rust writes the shortest decimal that
    // reads back as the same double, which is what makes this hold.
    for value in [
        1.0f64 / 3.0,
        f64::MIN_POSITIVE,
        f64::MAX,
        -0.0,
        1e-300,
        123_456_789.123_456_79,
    ] {
        let written = text(&Json::Number(value));
        let read = json::parse(written.as_bytes()).expect("readable");
        assert_eq!(
            read.as_number().map(f64::to_bits),
            Some(value.to_bits()),
            "{written}"
        );
    }
}

#[test]
fn a_number_that_is_not_finite_is_refused_rather_than_written() {
    let refused = Json::Number(f64::NAN)
        .to_bytes()
        .expect_err("JSON has no spelling for it");
    assert!(refused.contains("not finite"), "{refused}");
}

#[test]
fn a_key_that_appears_twice_is_refused() {
    // Both resolutions are somebody's convention. A header whose meaning
    // depends on which reader opened it is what the format version exists
    // against, so neither is taken.
    let refused = json::parse(b"{\"a\":1,\"a\":2}").expect_err("a repeated key");
    assert!(refused.contains("twice"), "{refused}");
}

#[test]
fn text_after_the_document_is_refused() {
    let refused = json::parse(b"{\"a\":1} {\"a\":2}").expect_err("two documents");
    assert!(refused.contains("characters follow it"), "{refused}");
}

#[test]
fn a_number_with_a_leading_zero_is_refused() {
    let refused = json::parse(b"[012]").expect_err("a leading zero");
    assert!(refused.contains("leading zero"), "{refused}");
}

#[test]
fn the_spellings_rust_accepts_and_json_does_not_are_refused() {
    // The near-miss. Each of these converts happily with a plain parse, so a
    // reader that scanned no grammar would accept a header carrying an infinite
    // axis value or a number with a sign JSON does not admit.
    for document in ["[inf]", "[NaN]", "[+1]", "[1e]", "[.5]", "[1.]"] {
        assert!(
            json::parse(document.as_bytes()).is_err(),
            "{document} was accepted"
        );
    }
}

#[test]
fn a_number_beyond_a_double_is_refused_rather_than_read_as_infinity() {
    let refused = json::parse(b"[1e400]").expect_err("out of range");
    assert!(refused.contains("outside the range"), "{refused}");
}

#[test]
fn an_escape_a_writer_elsewhere_emits_is_read() {
    // A writer in another language escapes a character outside the basic plane
    // as a surrogate pair without being asked to, and a reader that refused it
    // would reject a document its own writer would accept.
    let parsed = json::parse(b"[\"\\ud83d\\ude00 \\u00e5 \\/\"]").expect("readable");
    let items = parsed.as_array().expect("an array");
    assert_eq!(items[0].as_str(), Some("\u{1f600} \u{e5} /"));
}

#[test]
fn half_a_surrogate_pair_is_refused() {
    let refused = json::parse(b"[\"\\ud83d\"]").expect_err("a lone half");
    assert!(refused.contains("surrogate pair"), "{refused}");
}

#[test]
fn a_control_character_written_directly_into_a_string_is_refused() {
    let refused = json::parse(b"[\"a\tb\"]").expect_err("a raw tab");
    assert!(refused.contains("control character"), "{refused}");
}

#[test]
fn a_document_nested_past_the_bound_is_refused_rather_than_ending_the_process() {
    let deep = format!("{}1{}", "[".repeat(200), "]".repeat(200));
    let refused = json::parse(deep.as_bytes()).expect_err("too deep");
    assert!(refused.contains("nests deeper"), "{refused}");
}

#[test]
fn a_string_carries_every_character_it_has_to_escape_and_no_others() {
    let value = Json::String("\"\\\n\r\t\u{8}\u{c}\u{1}/\u{e5}".to_string());
    assert_eq!(text(&value), "\"\\\"\\\\\\n\\r\\t\\b\\f\\u0001/\u{e5}\"\n");
}

#[test]
fn a_document_that_is_not_text_is_refused() {
    let refused = json::parse(&[0xff, 0xfe]).expect_err("not UTF-8");
    assert!(refused.contains("not UTF-8"), "{refused}");
}

fn text(value: &Json) -> String {
    String::from_utf8(value.to_bytes().expect("writable")).expect("what was written is text")
}
