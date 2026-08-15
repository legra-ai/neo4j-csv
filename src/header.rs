//! Neo4j CSV header parsing.
//!
//! Parses the annotated header row into a sequence of [`ColumnKind`]
//! descriptors that drive row parsing.

use crate::error::Neo4jCsvError;

/// Describes what a single CSV column represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnKind {
    /// `:ID` — the node's format-native identifier.
    Id,
    /// `:LABEL` — semicolon-delimited node labels.
    Label,
    /// `:START_ID` — relationship source node identifier.
    StartId,
    /// `:END_ID` — relationship target node identifier.
    EndId,
    /// `:TYPE` — relationship type.
    RelType,
    /// A property column, optionally annotated with a type hint.
    Property {
        /// Property key name.
        name: String,
        /// Optional type hint (e.g. `"string"`, `"int"`).
        type_hint: Option<String>,
    },
}

/// Whether the header describes a node file or a relationship file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// Node file (has `:ID`).
    Node,
    /// Relationship file (has `:START_ID`, `:END_ID`, `:TYPE`).
    Relationship,
}

/// Parsed header: the column kinds and the file kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHeader {
    /// Column descriptors in header order.
    pub columns: Vec<ColumnKind>,
    /// Whether this is a node or relationship file.
    pub file_kind: FileKind,
}

/// Parse a Neo4j CSV header row into column descriptors.
///
/// # Errors
///
/// Returns [`Neo4jCsvError`] if the header is empty, ambiguous,
/// contains duplicates, or is not recognizable as a node or
/// relationship file.
pub fn parse_header(fields: &[&str]) -> Result<ParsedHeader, Neo4jCsvError> {
    if fields.is_empty() {
        return Err(Neo4jCsvError::EmptyHeader);
    }

    let mut columns = Vec::with_capacity(fields.len());
    let mut has_id = false;
    let mut has_start_id = false;
    let mut has_end_id = false;
    let mut has_type = false;
    let mut has_label = false;

    for field in fields {
        let trimmed = field.trim();
        let kind = match trimmed {
            ":ID" => {
                if has_id {
                    return Err(Neo4jCsvError::DuplicateColumn {
                        column: ":ID".to_owned(),
                    });
                }
                has_id = true;
                ColumnKind::Id
            }
            ":LABEL" => {
                if has_label {
                    return Err(Neo4jCsvError::DuplicateColumn {
                        column: ":LABEL".to_owned(),
                    });
                }
                has_label = true;
                ColumnKind::Label
            }
            ":START_ID" => {
                if has_start_id {
                    return Err(Neo4jCsvError::DuplicateColumn {
                        column: ":START_ID".to_owned(),
                    });
                }
                has_start_id = true;
                ColumnKind::StartId
            }
            ":END_ID" => {
                if has_end_id {
                    return Err(Neo4jCsvError::DuplicateColumn {
                        column: ":END_ID".to_owned(),
                    });
                }
                has_end_id = true;
                ColumnKind::EndId
            }
            ":TYPE" => {
                if has_type {
                    return Err(Neo4jCsvError::DuplicateColumn {
                        column: ":TYPE".to_owned(),
                    });
                }
                has_type = true;
                ColumnKind::RelType
            }
            other => parse_property_column(other),
        };
        columns.push(kind);
    }

    // Determine file kind.
    let is_node = has_id;
    let is_rel = has_start_id || has_end_id || has_type;

    if is_node && is_rel {
        return Err(Neo4jCsvError::AmbiguousHeader);
    }

    if is_node {
        return Ok(ParsedHeader {
            columns,
            file_kind: FileKind::Node,
        });
    }

    if is_rel {
        if !has_start_id {
            return Err(Neo4jCsvError::MissingColumn {
                column: ":START_ID".to_owned(),
            });
        }
        if !has_end_id {
            return Err(Neo4jCsvError::MissingColumn {
                column: ":END_ID".to_owned(),
            });
        }
        if !has_type {
            return Err(Neo4jCsvError::MissingColumn {
                column: ":TYPE".to_owned(),
            });
        }
        return Ok(ParsedHeader {
            columns,
            file_kind: FileKind::Relationship,
        });
    }

    Err(Neo4jCsvError::UnrecognizedFileType)
}

/// Parse a property column header like `"name:string"` or `"name"`.
fn parse_property_column(header: &str) -> ColumnKind {
    if let Some((name, hint)) = header.split_once(':') {
        ColumnKind::Property {
            name: name.to_owned(),
            type_hint: Some(hint.to_owned()),
        }
    } else {
        ColumnKind::Property {
            name: header.to_owned(),
            type_hint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_node_header() {
        let fields = vec![":ID", "name:string", "age:int", ":LABEL"];
        let parsed = parse_header(&fields).unwrap();
        assert_eq!(parsed.file_kind, FileKind::Node);
        assert_eq!(parsed.columns.len(), 4);
        assert_eq!(parsed.columns[0], ColumnKind::Id);
        assert_eq!(
            parsed.columns[1],
            ColumnKind::Property {
                name: "name".to_owned(),
                type_hint: Some("string".to_owned()),
            }
        );
        assert_eq!(
            parsed.columns[2],
            ColumnKind::Property {
                name: "age".to_owned(),
                type_hint: Some("int".to_owned()),
            }
        );
        assert_eq!(parsed.columns[3], ColumnKind::Label);
    }

    #[test]
    fn parse_relationship_header() {
        let fields = vec![":START_ID", ":END_ID", ":TYPE", "since:int"];
        let parsed = parse_header(&fields).unwrap();
        assert_eq!(parsed.file_kind, FileKind::Relationship);
        assert_eq!(parsed.columns.len(), 4);
        assert_eq!(parsed.columns[0], ColumnKind::StartId);
        assert_eq!(parsed.columns[1], ColumnKind::EndId);
        assert_eq!(parsed.columns[2], ColumnKind::RelType);
    }

    #[test]
    fn property_without_type_hint() {
        let fields = vec![":ID", "name"];
        let parsed = parse_header(&fields).unwrap();
        assert_eq!(
            parsed.columns[1],
            ColumnKind::Property {
                name: "name".to_owned(),
                type_hint: None,
            }
        );
    }

    #[test]
    fn empty_header_is_error() {
        let err = parse_header(&[]).unwrap_err();
        assert!(matches!(err, Neo4jCsvError::EmptyHeader));
    }

    #[test]
    fn ambiguous_header_is_error() {
        let fields = vec![":ID", ":START_ID", ":END_ID", ":TYPE"];
        let err = parse_header(&fields).unwrap_err();
        assert!(matches!(err, Neo4jCsvError::AmbiguousHeader));
    }

    #[test]
    fn missing_end_id_is_error() {
        let fields = vec![":START_ID", ":TYPE"];
        let err = parse_header(&fields).unwrap_err();
        assert!(matches!(
            err,
            Neo4jCsvError::MissingColumn { column } if column == ":END_ID"
        ));
    }

    #[test]
    fn duplicate_id_is_error() {
        let fields = vec![":ID", ":ID"];
        let err = parse_header(&fields).unwrap_err();
        assert!(matches!(
            err,
            Neo4jCsvError::DuplicateColumn { column } if column == ":ID"
        ));
    }

    #[test]
    fn unrecognized_file_type() {
        let fields = vec!["name:string", "age:int"];
        let err = parse_header(&fields).unwrap_err();
        assert!(matches!(err, Neo4jCsvError::UnrecognizedFileType));
    }
}
