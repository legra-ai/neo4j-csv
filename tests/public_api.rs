//! Public-API integration test: node and relationship files parse from the
//! shipped crate, whole-input and streaming.

use neo4j_csv::{
    Neo4jCsvError,
    Neo4jCsvRecord,
    Neo4jCsvStreamParser,
    parse_neo4j_csv,
    parse_neo4j_csv_with,
};

#[test]
fn node_file_parses_labels_and_typed_properties() {
    let input = ":ID,name:string,age:int,:LABEL\nalice,Alice,30,Person;Employee\n";
    let (nodes, relationships) = parse_neo4j_csv(input).expect("valid node file");
    assert!(relationships.is_empty());
    assert_eq!(nodes[0].id, "alice");
    assert_eq!(nodes[0].labels, ["Person", "Employee"]);
    assert_eq!(nodes[0].properties[1].name, "age");
    assert_eq!(nodes[0].properties[1].type_hint.as_deref(), Some("int"));
}

#[test]
fn relationship_file_streams_one_record_per_line() {
    let mut parser = Neo4jCsvStreamParser::new();
    assert!(
        parser
            .parse_line(":START_ID,:END_ID,:TYPE,since:int")
            .expect("header")
            .is_none()
    );
    let record = parser
        .parse_line("alice,bob,KNOWS,2020")
        .expect("row")
        .expect("record");
    let Neo4jCsvRecord::Edge(relationship) = record else {
        panic!("relationship expected");
    };
    assert_eq!(relationship.start_id, "alice");
    assert_eq!(relationship.end_id, "bob");
    assert_eq!(relationship.type_name, "KNOWS");
    assert_eq!(relationship.properties[0].value, "2020");

    let mut count = 0;
    parse_neo4j_csv_with(
        ":START_ID,:END_ID,:TYPE\na,b,X\nb,c,Y\n",
        |_| {},
        |_| count += 1,
    )
    .expect("valid relationship file");
    assert_eq!(count, 2);
}

#[test]
fn malformed_rows_are_typed_errors() {
    let error = parse_neo4j_csv(":ID,name:string\nalice\n").expect_err("field count mismatch");
    assert!(matches!(
        error,
        Neo4jCsvError::FieldCountMismatch {
            row: 1,
            found: 1,
            expected: 2
        }
    ));
    let error = parse_neo4j_csv("a,b,c\n1,2,3\n").expect_err("no annotations");
    assert!(matches!(error, Neo4jCsvError::UnrecognizedFileType));
}
