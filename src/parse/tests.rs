//! Inline unit tests for Neo4j CSV row parsing.

use crate::error::Neo4jCsvError;
use crate::parse::csv_line::parse_csv_line;
use crate::parse::entry::{
    parse_neo4j_csv,
    parse_neo4j_csv_with,
};
use crate::parse::record::Neo4jCsvRecord;
use crate::parse::stream::Neo4jCsvStreamParser;
use crate::types::Node;

#[test]
fn parse_basic_nodes() {
    let input = ":ID,name:string,age:int,:LABEL\nalice,Alice,30,Person\nbob,Bob,25,Person\n";
    let (nodes, edges) = parse_neo4j_csv(input).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 0);

    assert_eq!(nodes[0].id, "alice");
    assert_eq!(nodes[0].labels, vec!["Person"]);
    assert_eq!(nodes[0].properties.len(), 2);
    assert_eq!(nodes[0].properties[0].name, "name");
    assert_eq!(nodes[0].properties[0].value, "Alice");
    assert_eq!(nodes[0].properties[0].type_hint, Some("string".to_owned()));
}

#[test]
fn parse_multi_label() {
    let input = ":ID,:LABEL\nalice,Person;Employee\n";
    let (nodes, _) = parse_neo4j_csv(input).unwrap();
    assert_eq!(nodes[0].labels, vec!["Person", "Employee"]);
}

#[test]
fn parse_relationship_file() {
    let input = ":START_ID,:END_ID,:TYPE,since:int\nalice,bob,KNOWS,2020\n";
    let (nodes, edges) = parse_neo4j_csv(input).unwrap();
    assert_eq!(nodes.len(), 0);
    assert_eq!(edges.len(), 1);

    assert_eq!(edges[0].start_id, "alice");
    assert_eq!(edges[0].end_id, "bob");
    assert_eq!(edges[0].type_name, "KNOWS");
    assert_eq!(edges[0].properties.len(), 1);
    assert_eq!(edges[0].properties[0].name, "since");
    assert_eq!(edges[0].properties[0].value, "2020");
}

#[test]
fn parse_quoted_fields() {
    let input = ":ID,name:string,:LABEL\nalice,\"Smith, Alice\",Person\n";
    let (nodes, _) = parse_neo4j_csv(input).unwrap();
    assert_eq!(nodes[0].properties[0].value, "Smith, Alice");
}

#[test]
fn parse_escaped_quotes() {
    let input = ":ID,name:string,:LABEL\nalice,\"She said \"\"hi\"\"\",Person\n";
    let (nodes, _) = parse_neo4j_csv(input).unwrap();
    assert_eq!(nodes[0].properties[0].value, "She said \"hi\"");
}

#[test]
fn parse_empty_values() {
    let input = ":ID,name:string,age:int,:LABEL\nalice,,30,Person\n";
    let (nodes, _) = parse_neo4j_csv(input).unwrap();
    assert_eq!(nodes[0].properties[0].value, "");
    assert_eq!(nodes[0].properties[1].value, "30");
}

#[test]
fn skip_empty_lines() {
    let input = ":ID,:LABEL\nalice,Person\n\nbob,Person\n";
    let (nodes, _) = parse_neo4j_csv(input).unwrap();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn field_count_mismatch_error() {
    let input = ":ID,name:string,:LABEL\nalice,Alice\n";
    let err = parse_neo4j_csv(input).unwrap_err();
    assert!(matches!(
        err,
        Neo4jCsvError::FieldCountMismatch {
            row: 1,
            found: 2,
            expected: 3
        }
    ));
}

#[test]
fn empty_input_error() {
    let err = parse_neo4j_csv("").unwrap_err();
    assert!(matches!(err, Neo4jCsvError::EmptyHeader));
}

#[test]
fn csv_line_parser_basic() {
    let fields = parse_csv_line("a,b,c");
    assert_eq!(fields, vec!["a", "b", "c"]);
}

#[test]
fn csv_line_parser_quoted() {
    let fields = parse_csv_line("\"hello, world\",b");
    assert_eq!(fields, vec!["hello, world", "b"]);
}

#[test]
fn csv_line_parser_escaped_quote() {
    let fields = parse_csv_line("\"a\"\"b\",c");
    assert_eq!(fields, vec!["a\"b", "c"]);
}

#[test]
fn csv_line_parser_empty_fields() {
    let fields = parse_csv_line(",b,");
    assert_eq!(fields, vec!["", "b", ""]);
}

#[test]
fn streaming_parse() {
    let input = ":ID,:LABEL\nalice,Person\nbob,Person\n";
    let mut count = 0;
    parse_neo4j_csv_with(input, |_node| count += 1, |_edge| {}).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn stateful_parser_emits_nodes() {
    let mut parser = Neo4jCsvStreamParser::new();
    assert!(!parser.has_header());
    assert!(parser.parse_line(":ID,:LABEL").unwrap().is_none());
    assert!(parser.has_header());

    let first = parser.parse_line("alice,Person").unwrap().unwrap();
    let second = parser.parse_line("bob,Person").unwrap().unwrap();

    assert_eq!(
        first,
        Neo4jCsvRecord::Node(Node {
            id: "alice".to_owned(),
            labels: vec!["Person".to_owned()],
            properties: Vec::new(),
        })
    );
    assert_eq!(
        second,
        Neo4jCsvRecord::Node(Node {
            id: "bob".to_owned(),
            labels: vec!["Person".to_owned()],
            properties: Vec::new(),
        })
    );
}

#[test]
fn stateful_parser_counts_blank_lines_in_row_numbers() {
    let mut parser = Neo4jCsvStreamParser::new();
    parser.parse_line(":ID,name:string,:LABEL").unwrap();
    assert!(parser.parse_line("").unwrap().is_none());

    let err = parser.parse_line("alice,Alice").unwrap_err();
    assert!(matches!(
        err,
        Neo4jCsvError::FieldCountMismatch {
            row: 2,
            found: 2,
            expected: 3
        }
    ));
}
