# media-typer

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

[![crates.io](https://img.shields.io/crates/v/media-typer.svg)](https://crates.io/crates/media-typer)
[![docs.rs](https://docs.rs/media-typer/badge.svg)](https://docs.rs/media-typer)
[![CI](https://github.com/trananhtung/media-typer/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/media-typer/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/media-typer.svg)](#license)

**Parse and format media types per [RFC 6838](https://www.rfc-editor.org/rfc/rfc6838).**

`media-typer` splits a media type — `type/subtype+suffix` — into its parts and validates
each against the RFC 6838 `restricted-name` grammar, and re-assembles them. A faithful Rust
port of the widely-used [`media-typer`](https://www.npmjs.com/package/media-typer) npm
package (v2), used throughout the Node HTTP stack (`type-is`, `body-parser`, …).

This is the strict, lower-level media-type grammar — it does **not** parse parameters
(`; charset=utf-8`). For full `Content-Type` header handling (with parameters), see the
[`content-type`](https://crates.io/crates/content-type) crate.

- **Zero dependencies**
- **`#![no_std]`**
- Differential-tested against the reference `media-typer` implementation

## Install

```toml
[dependencies]
media-typer = "0.1"
```

## Usage

```rust
use media_typer::{parse, format, test, MediaType};

// Parse splits out type / subtype / suffix and lower-cases them.
let mt = parse("application/vnd.api+json").unwrap();
assert_eq!(mt.type_, "application");
assert_eq!(mt.subtype, "vnd.api");
assert_eq!(mt.suffix.as_deref(), Some("json"));

assert_eq!(parse("IMAGE/SVG+XML").unwrap(), MediaType::new("image", "svg", Some("xml")));

// Format validates each part and re-assembles.
assert_eq!(format(&MediaType::new("text", "html", None::<&str>)).unwrap(), "text/html");

// Test reports validity (no parameters allowed).
assert!(test("application/json"));
assert!(!test("application/json; charset=utf-8"));
```

## Behavior notes

- `parse` lower-cases the `type`, `subtype`, and `suffix`, and takes the suffix from after
  the **last** `+` in the subtype (`application/a+b+c` → subtype `a+b`, suffix `c`).
- A media type with parameters is rejected — `text/html; charset=utf-8` is **not** a valid
  bare media type. Strip parameters first if you have a full `Content-Type` header.
- Names follow `restricted-name`: an ASCII letter or digit, then up to 126 more of
  `A-Za-z0-9!#$&^_-` (subtypes and suffixes also allow `.` and `+`), so 1–127 characters.
- As in the reference, a degenerate subtype like `x++` parses to subtype `x+` with an
  **empty** suffix `Some("")`, which then fails to `format` again (an empty suffix is not a
  valid name). This faithfully mirrors the npm package.

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/./commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
