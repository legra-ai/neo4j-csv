# neo4j-csv

[![Crates.io](https://img.shields.io/crates/v/neo4j-csv.svg)](https://crates.io/crates/neo4j-csv)
[![Downloads](https://img.shields.io/crates/d/neo4j-csv.svg)](https://crates.io/crates/neo4j-csv)
[![Documentation](https://docs.rs/neo4j-csv/badge.svg)](https://docs.rs/neo4j-csv)
[![CI](https://github.com/legra-ai/neo4j-csv/actions/workflows/ci.yml/badge.svg)](https://github.com/legra-ai/neo4j-csv/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/legra-ai/neo4j-csv)

An owned, streaming parser for Neo4j’s annotated CSV import format.

Neo4j’s bulk-import CSV files use annotations such as `:ID`, `:LABEL`,
`:START_ID`, `:END_ID`, and `:TYPE` in the header row. `neo4j-csv` parses
those headers and rows into small, format-neutral records without depending
on a database driver, graph store, RDF model, or application framework.

## Why this crate

The parser is intentionally split into two layers:

- header parsing identifies node, relationship, and property columns;
- row parsing converts each line into an owned [`Node`] or [`Relationship`].

The [`Neo4jCsvStreamParser`] API processes one physical line at a time. This
keeps memory bounded for large imports and lets callers decide where records
go. The convenience [`parse_neo4j_csv`] function is available when collecting
the complete result is appropriate.

## Quick start

```rust
use neo4j_csv::{Node, parse_neo4j_csv};

let input = ":ID,name:string,:LABEL\nalice,Alice,Person\n";
let (nodes, relationships) = parse_neo4j_csv(input)?;

assert!(relationships.is_empty());
assert_eq!(nodes[0].id, "alice");
assert_eq!(nodes[0].labels, ["Person"]);
assert_eq!(nodes[0].properties[0].name, "name");
assert_eq!(nodes[0].properties[0].value, "Alice");

# Ok::<(), neo4j_csv::Neo4jCsvError>(())
```

## Streaming large imports

Use [`parse_neo4j_csv_with`] to handle records as they are parsed:

```rust
use neo4j_csv::parse_neo4j_csv_with;

let input = ":START_ID,:END_ID,:TYPE\nalice,bob,KNOWS\n";
let mut relationships = 0;

parse_neo4j_csv_with(input, |_node| {}, |_relationship| {
    relationships += 1;
})?;

assert_eq!(relationships, 1);
# Ok::<(), neo4j_csv::Neo4jCsvError>(())
```

For input that is already available as a string, this API avoids retaining a
second collection of parsed records. [`Neo4jCsvStreamParser`] is the lower
level API for callers that read from a file, socket, or other source in their
own I/O layer.

## Supported annotations

- `:ID` for node identifiers;
- `:LABEL` for semicolon-separated node labels;
- `:START_ID` and `:END_ID` for relationship endpoints;
- `:TYPE` for relationship types;
- `name` for an untyped property;
- `age:int` or `created:string` for a property with a type hint;
- RFC 4180 quoting, including commas and escaped double quotes inside fields.

Malformed headers and rows produce the typed [`Neo4jCsvError`] error.

## Scope

This crate parses Neo4j’s annotated CSV representation. It does not connect
to Neo4j, resolve IDs across files, validate property type values, or convert
records into another graph or RDF model.

## License

Licensed under either of:

- MIT license ([`LICENSE-MIT`](LICENSE-MIT) or <https://opensource.org/licenses/MIT>);
- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>).

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
