//! Integration tests exercising the public API of `media-typer`.

use media_typer::{format, parse, test, Error, MediaType};

#[test]
fn round_trips_common_types() {
    for s in [
        "text/plain",
        "application/json",
        "image/png",
        "application/vnd.api+json",
        "application/ld+json",
        "image/svg+xml",
    ] {
        let mt = parse(s).unwrap();
        assert_eq!(format(&mt).unwrap(), s, "round-trip failed for {s}");
    }
}

#[test]
fn parse_then_inspect() {
    let mt = parse("application/vnd.github.v3+json").unwrap();
    assert_eq!(mt.type_, "application");
    assert_eq!(mt.subtype, "vnd.github.v3");
    assert_eq!(mt.suffix.as_deref(), Some("json"));
}

#[test]
fn rejects_parameters_and_garbage() {
    assert!(parse("text/html; charset=utf-8").is_err());
    assert!(parse("just-text").is_err());
    assert!(!test("text/html;q=1"));
}

#[test]
fn format_validates_each_part() {
    assert_eq!(
        format(&MediaType::new("bad type", "json", None::<&str>)),
        Err(Error::InvalidType("bad type".into()))
    );
    assert_eq!(
        format(&MediaType::new("application", "bad subtype", None::<&str>)),
        Err(Error::InvalidSubtype("bad subtype".into()))
    );
    assert_eq!(
        format(&MediaType::new("application", "json", Some("not valid"))),
        Err(Error::InvalidSuffix("not valid".into()))
    );
}
