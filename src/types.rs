//! Format-neutral records produced by the Neo4j CSV parser.

/// A node parsed from a Neo4j node CSV file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The format-native node identifier.
    pub id: String,
    /// Semicolon-delimited labels from the `:LABEL` column.
    pub labels: Vec<String>,
    /// Properties attached to the node.
    pub properties: Vec<Property>,
}

/// A relationship parsed from a Neo4j relationship CSV file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    /// The format-native source node identifier.
    pub start_id: String,
    /// The format-native target node identifier.
    pub end_id: String,
    /// The relationship type from the `:TYPE` column.
    pub type_name: String,
    /// Properties attached to the relationship.
    pub properties: Vec<Property>,
}

/// A string-valued property with an optional Neo4j type hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    /// The property key.
    pub name: String,
    /// The property value as it appeared after CSV unquoting.
    pub value: String,
    /// The optional type hint from an annotated header such as `age:int`.
    pub type_hint: Option<String>,
}
