//! Stateful line-by-line streaming parser.

use crate::error::Neo4jCsvError;

use crate::header::{FileKind, ParsedHeader, parse_header};
use crate::parse::build::{build_edge, build_node};
use crate::parse::csv_line::parse_csv_line;
use crate::parse::record::Neo4jCsvRecord;

/// Stateful line-by-line Neo4j CSV parser.
#[derive(Debug, Default)]
pub struct Neo4jCsvStreamParser {
    header: Option<ParsedHeader>,
    row: usize,
}

impl Neo4jCsvStreamParser {
    /// Create a new streaming parser with no header loaded yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return whether the header row has been parsed.
    #[must_use]
    pub fn has_header(&self) -> bool {
        self.header.is_some()
    }

    /// Parse one physical CSV line.
    ///
    /// The first line is treated as the header. Subsequent lines yield
    /// zero or one parsed records.
    ///
    /// # Errors
    ///
    /// Returns [`Neo4jCsvError`] when the header or a data row is
    /// malformed.
    pub fn parse_line(&mut self, line: &str) -> Result<Option<Neo4jCsvRecord>, Neo4jCsvError> {
        let line = line.strip_suffix('\r').unwrap_or(line);

        let Some(header) = self.header.as_ref() else {
            let header_fields = parse_csv_line(line);
            let header_refs: Vec<&str> = header_fields.iter().map(String::as_str).collect();
            self.header = Some(parse_header(&header_refs)?);
            return Ok(None);
        };

        self.row += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let fields = parse_csv_line(trimmed);
        if fields.len() != header.columns.len() {
            return Err(Neo4jCsvError::FieldCountMismatch {
                row: self.row,
                found: fields.len(),
                expected: header.columns.len(),
            });
        }

        let record = match header.file_kind {
            FileKind::Node => Neo4jCsvRecord::Node(build_node(&header.columns, &fields)),
            FileKind::Relationship => Neo4jCsvRecord::Edge(build_edge(&header.columns, &fields)),
        };
        Ok(Some(record))
    }
}
