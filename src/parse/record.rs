//! The parsed-record enum yielded by streaming parsing.

use crate::types::{
    Node,
    Relationship,
};

/// One parsed Neo4j CSV record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Neo4jCsvRecord {
    /// A node row from a node CSV file.
    Node(Node),
    /// A relationship row from a relationship CSV file.
    Edge(Relationship),
}
