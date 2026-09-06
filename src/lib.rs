#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
//! Streaming parser for Neo4j’s annotated CSV import format.
//!
//! The crate parses node and relationship files into owned, format-neutral
//! records. It has no database, graph-model, or application-specific
//! dependencies.

mod error;

pub mod header;
pub mod parse;
pub mod types;

pub use error::Neo4jCsvError;
pub use parse::{
    Neo4jCsvRecord,
    Neo4jCsvStreamParser,
    parse_neo4j_csv,
    parse_neo4j_csv_with,
};
pub use types::{
    Node,
    Property,
    Relationship,
};
