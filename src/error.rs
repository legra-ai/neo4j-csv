//! Neo4j CSV parser error types.

/// Errors from parsing Neo4j CSV files.
#[derive(Debug, thiserror::Error)]
pub enum Neo4jCsvError {
    /// The CSV header is missing or empty.
    #[error("Neo4j CSV: empty or missing header")]
    EmptyHeader,

    /// The CSV header is ambiguous (has both node and relationship
    /// columns).
    #[error("Neo4j CSV: ambiguous header — contains both :ID and :START_ID")]
    AmbiguousHeader,

    /// A required header column is missing.
    #[error("Neo4j CSV: missing required column: {column}")]
    MissingColumn {
        /// The missing column name.
        column: String,
    },

    /// A duplicate header column was found.
    #[error("Neo4j CSV: duplicate column: {column}")]
    DuplicateColumn {
        /// The duplicate column name.
        column: String,
    },

    /// A data row has the wrong number of fields.
    #[error("Neo4j CSV: row {row} has {found} fields, expected {expected}")]
    FieldCountMismatch {
        /// 1-based row number (excluding header).
        row: usize,
        /// Number of fields found.
        found: usize,
        /// Expected number of fields.
        expected: usize,
    },

    /// The header describes neither a node file nor a relationship
    /// file.
    #[error(
        "Neo4j CSV: header is neither a node file (:ID) nor a relationship file (:START_ID + :END_ID + :TYPE)"
    )]
    UnrecognizedFileType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_empty_header() {
        let err = Neo4jCsvError::EmptyHeader;
        assert_eq!(err.to_string(), "Neo4j CSV: empty or missing header");
    }

    #[test]
    fn display_field_count_mismatch() {
        let err = Neo4jCsvError::FieldCountMismatch {
            row: 3,
            found: 4,
            expected: 5,
        };
        assert_eq!(err.to_string(), "Neo4j CSV: row 3 has 4 fields, expected 5");
    }
}
