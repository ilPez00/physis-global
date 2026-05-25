use crate::models::*;

pub const HUMAN_ONTOLOGY_NAME: &str = "praxis_ontology";
pub const HUMAN_TYPE: &str = "human";

pub fn parse_ontology_entry(entry: &OntologyEntry) -> DomainDef {
    DomainDef {
        name: entry.name.clone(),
        category: entry.category.clone(),
        unit: entry.unit.clone(),
        hints: entry.hints.clone(),
    }
}
