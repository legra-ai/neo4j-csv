//! Conversions into the format-neutral `property-graph-model` records.
//!
//! Enabled by the `property-graph-model` feature. Each parsed record
//! maps field-for-field, so a streaming import pipeline can hand the
//! parser's output straight to any consumer of `PgNode` / `PgEdge`.

use property_graph_model::{
    PgEdge,
    PgNode,
    PgProperty,
};

use crate::types::{
    Node,
    Property,
    Relationship,
};

impl From<Property> for PgProperty {
    fn from(property: Property) -> Self {
        Self {
            key: property.name,
            value: property.value,
            type_hint: property.type_hint,
        }
    }
}

impl From<Node> for PgNode {
    fn from(node: Node) -> Self {
        Self {
            source_id: node.id,
            labels: node.labels,
            properties: node.properties.into_iter().map(PgProperty::from).collect(),
        }
    }
}

impl From<Relationship> for PgEdge {
    fn from(relationship: Relationship) -> Self {
        Self {
            source_id: relationship.start_id,
            target_id: relationship.end_id,
            rel_type: relationship.type_name,
            properties: relationship
                .properties
                .into_iter()
                .map(PgProperty::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use property_graph_model::{
        PgEdge,
        PgNode,
    };

    use crate::parse_neo4j_csv;

    #[test]
    fn parsed_records_convert_field_for_field() {
        let input = ":ID,name:string,age:int,:LABEL\n1,Alice,30,Person;Employee\n";
        let (nodes, _) = parse_neo4j_csv(input).expect("parse");
        let node = PgNode::from(nodes.into_iter().next().expect("one node"));
        assert_eq!(node.source_id, "1");
        assert_eq!(node.labels, vec!["Person", "Employee"]);
        assert_eq!(node.properties.len(), 2);
        assert_eq!(node.properties[0].key, "name");
        assert_eq!(node.properties[1].type_hint.as_deref(), Some("int"));

        let rels = ":START_ID,:END_ID,:TYPE,since:int\n1,2,KNOWS,2020\n";
        let (_, relationships) = parse_neo4j_csv(rels).expect("parse");
        let edge = PgEdge::from(relationships.into_iter().next().expect("one relationship"));
        assert_eq!(edge.source_id, "1");
        assert_eq!(edge.target_id, "2");
        assert_eq!(edge.rel_type, "KNOWS");
        assert_eq!(edge.properties[0].value, "2020");
    }
}
