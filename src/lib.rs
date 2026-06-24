//! # media-typer — RFC 6838 media type parsing and formatting
//!
//! Parse and format media types — `type/subtype+suffix` — per the
//! [RFC 6838](https://www.rfc-editor.org/rfc/rfc6838) grammar, splitting out the `type`,
//! `subtype`, and structured-syntax `suffix`. A faithful Rust port of the widely-used
//! [`media-typer`](https://www.npmjs.com/package/media-typer) npm package (v2), used
//! throughout the Node HTTP stack (`type-is`, `body-parser`, …).
//!
//! This is the strict, lower-level media-type grammar — it does **not** parse parameters
//! (`; charset=utf-8`); for full `Content-Type` header handling see the `content-type`
//! crate. **Zero dependencies** and `#![no_std]`.
//!
//! ```
//! use media_typer::{parse, format, test, MediaType};
//!
//! let mt = parse("application/vnd.api+json").unwrap();
//! assert_eq!(mt.type_, "application");
//! assert_eq!(mt.subtype, "vnd.api");
//! assert_eq!(mt.suffix.as_deref(), Some("json"));
//!
//! // The type, subtype, and suffix are lower-cased on parse.
//! assert_eq!(parse("IMAGE/SVG+XML").unwrap(), MediaType::new("image", "svg", Some("xml")));
//!
//! // `format` re-assembles and validates.
//! assert_eq!(format(&MediaType::new("text", "html", None::<&str>)).unwrap(), "text/html");
//!
//! // `test` reports whether a string is a valid (parameter-free) media type.
//! assert!(test("application/json"));
//! assert!(!test("application/json; charset=utf-8"));
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/media-typer/0.1.0")]

extern crate alloc;

use alloc::string::{String, ToString};

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// A parsed media type: `type/subtype` with an optional structured-syntax `suffix`.
///
/// The `suffix` is `None` when the subtype has no `+`, and `Some(_)` otherwise — note it
/// can be `Some("")` for a degenerate subtype like `x++` (matching the reference), which
/// is why round-tripping such a value back through [`format`] fails.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaType {
    /// The top-level type, e.g. `application` (lower-cased by [`parse`]).
    pub type_: String,
    /// The subtype, e.g. `vnd.api` (lower-cased by [`parse`], suffix removed).
    pub subtype: String,
    /// The structured-syntax suffix after the last `+`, e.g. `json`.
    pub suffix: Option<String>,
}

impl MediaType {
    /// Construct a [`MediaType`] from its parts (without validation — use [`format`] to
    /// validate and render).
    pub fn new(
        type_: impl Into<String>,
        subtype: impl Into<String>,
        suffix: Option<impl Into<String>>,
    ) -> Self {
        Self {
            type_: type_.into(),
            subtype: subtype.into(),
            suffix: suffix.map(Into::into),
        }
    }

    /// Validate and render this media type to a string. See [`format`].
    ///
    /// # Errors
    /// Returns an error if the `type_`, `subtype`, or `suffix` is not a valid RFC 6838 name.
    pub fn format(&self) -> Result<String, Error> {
        format(self)
    }
}

/// The error type for invalid media types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A string passed to [`parse`] is not a valid media type.
    InvalidMediaType(String),
    /// The `type` part is not a valid RFC 6838 name.
    InvalidType(String),
    /// The `subtype` part is not a valid RFC 6838 name.
    InvalidSubtype(String),
    /// The `suffix` part is not a valid RFC 6838 name.
    InvalidSuffix(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidMediaType(s) => write!(f, "invalid media type: {s}"),
            Error::InvalidType(s) => write!(f, "invalid type: {s}"),
            Error::InvalidSubtype(s) => write!(f, "invalid subtype: {s}"),
            Error::InvalidSuffix(s) => write!(f, "invalid suffix: {s}"),
        }
    }
}

impl core::error::Error for Error {}

/// Parse a media type string into its [`MediaType`] parts.
///
/// The `type`, `subtype`, and `suffix` are lower-cased; the `suffix` is taken from after
/// the **last** `+` in the subtype. Parameters (`; key=value`) are **not** accepted.
///
/// # Errors
/// Returns [`Error::InvalidMediaType`] if `media_type` is not a valid RFC 6838 media type.
///
/// ```
/// # use media_typer::parse;
/// assert_eq!(parse("text/plain").unwrap().subtype, "plain");
/// assert!(parse("text/html; charset=utf-8").is_err());
/// ```
pub fn parse(media_type: &str) -> Result<MediaType, Error> {
    let (type_part, subtype_part) = match media_type.split_once('/') {
        Some((t, sub)) if is_type_name(t) && is_subtype_name(sub) => (t, sub),
        _ => return Err(Error::InvalidMediaType(media_type.to_string())),
    };
    let type_ = type_part.to_ascii_lowercase();
    let mut subtype = subtype_part.to_ascii_lowercase();
    let suffix = subtype.rfind('+').map(|idx| {
        let suffix = subtype[idx + 1..].to_string();
        subtype.truncate(idx);
        suffix
    });
    Ok(MediaType {
        type_,
        subtype,
        suffix,
    })
}

/// Validate the parts of a [`MediaType`] and render it as `type/subtype[+suffix]`.
///
/// # Errors
/// Returns [`Error::InvalidType`], [`Error::InvalidSubtype`], or [`Error::InvalidSuffix`]
/// if the corresponding part is not a valid RFC 6838 name. Note a `Some("")` suffix is
/// invalid.
///
/// ```
/// # use media_typer::{format, MediaType};
/// assert_eq!(
///     format(&MediaType::new("application", "vnd.api", Some("json"))).unwrap(),
///     "application/vnd.api+json"
/// );
/// ```
pub fn format(media_type: &MediaType) -> Result<String, Error> {
    if !is_type_name(&media_type.type_) {
        return Err(Error::InvalidType(media_type.type_.clone()));
    }
    if !is_subtype_name(&media_type.subtype) {
        return Err(Error::InvalidSubtype(media_type.subtype.clone()));
    }
    let mut out = String::with_capacity(media_type.type_.len() + media_type.subtype.len() + 8);
    out.push_str(&media_type.type_);
    out.push('/');
    out.push_str(&media_type.subtype);
    if let Some(suffix) = &media_type.suffix {
        if !is_type_name(suffix) {
            return Err(Error::InvalidSuffix(suffix.clone()));
        }
        out.push('+');
        out.push_str(suffix);
    }
    Ok(out)
}

/// Report whether `media_type` is a valid (parameter-free) RFC 6838 media type.
///
/// ```
/// # use media_typer::test;
/// assert!(test("image/png"));
/// assert!(!test("image png"));
/// ```
#[must_use]
pub fn test(media_type: &str) -> bool {
    match media_type.split_once('/') {
        Some((type_, subtype)) => is_type_name(type_) && is_subtype_name(subtype),
        None => false,
    }
}

/// `restricted-name` for a type/suffix name: `^[A-Za-z0-9][A-Za-z0-9!#$&^_-]{0,126}$`.
fn is_type_name(s: &str) -> bool {
    let bytes = s.as_bytes();
    match bytes.split_first() {
        Some((first, rest)) if first.is_ascii_alphanumeric() && rest.len() <= 126 => {
            rest.iter().all(|&b| is_type_char(b))
        }
        _ => false,
    }
}

/// `restricted-name` for a subtype name (also allows `.` and `+`):
/// `^[A-Za-z0-9][A-Za-z0-9!#$&^_.+-]{0,126}$`.
fn is_subtype_name(s: &str) -> bool {
    let bytes = s.as_bytes();
    match bytes.split_first() {
        Some((first, rest)) if first.is_ascii_alphanumeric() && rest.len() <= 126 => {
            rest.iter().all(|&b| is_subtype_char(b))
        }
        _ => false,
    }
}

fn is_type_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'-')
}

fn is_subtype_char(b: u8) -> bool {
    is_type_char(b) || b == b'.' || b == b'+'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        assert_eq!(
            parse("application/json").unwrap(),
            MediaType::new("application", "json", None::<&str>)
        );
        assert_eq!(
            parse("application/vnd.api+json").unwrap(),
            MediaType::new("application", "vnd.api", Some("json"))
        );
    }

    #[test]
    fn parse_lowercases() {
        assert_eq!(
            parse("IMAGE/SVG+XML").unwrap(),
            MediaType::new("image", "svg", Some("xml"))
        );
    }

    #[test]
    fn parse_suffix_at_last_plus() {
        let mt = parse("application/a+b+c").unwrap();
        assert_eq!(mt.subtype, "a+b");
        assert_eq!(mt.suffix.as_deref(), Some("c"));
    }

    #[test]
    fn parse_degenerate_empty_suffix() {
        let mt = parse("application/x++").unwrap();
        assert_eq!(mt.subtype, "x+");
        assert_eq!(mt.suffix.as_deref(), Some(""));
        // ...and it cannot be re-formatted, like the reference.
        assert_eq!(mt.format(), Err(Error::InvalidSuffix(String::new())));
    }

    #[test]
    fn parse_rejects_invalid() {
        for bad in [
            "bogus",
            "a/b/c",
            "",
            ".a/b",
            "a/.b",
            "text/html; q=1",
            "a /b",
            "/",
        ] {
            assert!(parse(bad).is_err(), "{bad:?} should be invalid");
        }
    }

    #[test]
    fn test_function() {
        assert!(test("text/html"));
        assert!(test("application/vnd.api+json"));
        assert!(!test("text/html;x=1"));
        assert!(!test("text/html "));
        assert!(!test("text"));
    }

    #[test]
    fn format_basic_and_errors() {
        assert_eq!(
            format(&MediaType::new("application", "json", None::<&str>)).unwrap(),
            "application/json"
        );
        assert_eq!(
            format(&MediaType::new("application", "vnd.api", Some("xml"))).unwrap(),
            "application/vnd.api+xml"
        );
        assert_eq!(
            format(&MediaType::new("a/b", "c", None::<&str>)),
            Err(Error::InvalidType("a/b".to_string()))
        );
        assert_eq!(
            format(&MediaType::new("a", "b", Some(""))),
            Err(Error::InvalidSuffix(String::new()))
        );
    }

    #[test]
    fn length_limits() {
        let ok = alloc::format!("a/{}", "b".repeat(127)); // subtype 127 chars (max)
        assert!(test(&ok));
        let too_long = alloc::format!("a/{}", "b".repeat(128)); // subtype 128 chars
        assert!(!test(&too_long));
    }
}
