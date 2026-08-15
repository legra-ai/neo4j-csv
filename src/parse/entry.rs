//! Top-level entry points for parsing a complete Neo4j CSV input.

use crate::error::Neo4jCsvError;
use crate::types::{Node, Relationship};

use crate::parse::record::Neo4jCsvRecord;
use crate::parse::stream::Neo4jCsvStreamParser;

/// Parse a Neo4j CSV input into nodes and edges.
///
/// The first line is the header; subsequent lines are data rows.
/// Returns all parsed nodes and edges.
///
/// # Errors
///
/// Returns [`Neo4jCsvError`] on header or row parse errors.
pub fn parse_neo4j_csv(input: &str) -> Result<(Vec<Node>, Vec<Relationship>), Neo4jCsvError> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    parse_neo4j_csv_with(input, |node| nodes.push(node), |edge| edges.push(edge))?;
    Ok((nodes, edges))
}

/// Parse a Neo4j CSV input with callbacks for each node and edge.
///
/// This is the streaming variant: each parsed entity is handed to
/// the appropriate callback immediately.
///
/// # Errors
///
/// Returns [`Neo4jCsvError`] on header or row parse errors.
pub fn parse_neo4j_csv_with<F, G>(
    input: &str,
    mut on_node: F,
    mut on_edge: G,
) -> Result<(), Neo4jCsvError>
where
    F: FnMut(Node),
    G: FnMut(Relationship),
{
    let mut lines = input.lines();
    let Some(header_line) = lines.next() else {
        return Err(Neo4jCsvError::EmptyHeader);
    };

    let mut parser = Neo4jCsvStreamParser::new();
    parser.parse_line(header_line)?;

    for line in lines {
        if let Some(record) = parser.parse_line(line)? {
            match record {
                Neo4jCsvRecord::Node(node) => on_node(node),
                Neo4jCsvRecord::Edge(edge) => on_edge(edge),
            }
        }
    }

    Ok(())
}
