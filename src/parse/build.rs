//! Construction of parsed records from data rows.

use crate::header::ColumnKind;
use crate::types::{
    Node,
    Property,
    Relationship,
};

/// Build a [`Node`] from a data row and column descriptors.
pub(super) fn build_node(columns: &[ColumnKind], fields: &[String]) -> Node {
    let mut id = String::new();
    let mut labels = Vec::new();
    let mut properties = Vec::new();

    for (col, value) in columns.iter().zip(fields.iter()) {
        match col {
            ColumnKind::Id => {
                id.clone_from(value);
            }
            ColumnKind::Label => {
                // Labels are semicolon-delimited.
                for label in value.split(';') {
                    let trimmed = label.trim();
                    if !trimmed.is_empty() {
                        labels.push(trimmed.to_owned());
                    }
                }
            }
            ColumnKind::Property { name, type_hint } => {
                properties.push(Property {
                    name: name.clone(),
                    value: value.clone(),
                    type_hint: type_hint.clone(),
                });
            }
            // Node files shouldn't have these, but we ignore them.
            ColumnKind::StartId | ColumnKind::EndId | ColumnKind::RelType => {}
        }
    }

    Node {
        id,
        labels,
        properties,
    }
}

/// Build a [`Relationship`] from a data row and column descriptors.
pub(super) fn build_edge(columns: &[ColumnKind], fields: &[String]) -> Relationship {
    let mut start_id = String::new();
    let mut end_id = String::new();
    let mut type_name = String::new();
    let mut properties = Vec::new();

    for (col, value) in columns.iter().zip(fields.iter()) {
        match col {
            ColumnKind::StartId => {
                start_id.clone_from(value);
            }
            ColumnKind::EndId => {
                end_id.clone_from(value);
            }
            ColumnKind::RelType => {
                type_name.clone_from(value);
            }
            ColumnKind::Property { name, type_hint } => {
                properties.push(Property {
                    name: name.clone(),
                    value: value.clone(),
                    type_hint: type_hint.clone(),
                });
            }
            // Relationship files shouldn't have these.
            ColumnKind::Id | ColumnKind::Label => {}
        }
    }

    Relationship {
        start_id,
        end_id,
        type_name,
        properties,
    }
}
