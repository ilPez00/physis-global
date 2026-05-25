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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_exist() {
        assert_eq!(HUMAN_ONTOLOGY_NAME, "praxis_ontology");
        assert_eq!(HUMAN_TYPE, "human");
    }

    #[test]
    fn test_parse_ontology_entry() {
        let entry = OntologyEntry {
            name: "logic".into(),
            category: Some("reasoning".into()),
            domain: "reasoning".into(),
            mode: "COHERENCE".into(),
            axis_kind: "epistemic".into(),
            axis_name: "formal".into(),
            unit: "tokens".into(),
            hints: vec!["therefore".into(), "implies".into()],
        };
        let def = parse_ontology_entry(&entry);
        assert_eq!(def.name, "logic");
        assert_eq!(def.category.as_deref(), Some("reasoning"));
        assert_eq!(def.unit, "tokens");
        assert_eq!(def.hints, vec!["therefore", "implies"]);
    }

    #[test]
    fn test_parse_ontology_entry_no_category() {
        let entry = OntologyEntry {
            name: "empty".into(),
            category: None,
            domain: "generic".into(),
            mode: "PLAN".into(),
            axis_kind: "none".into(),
            axis_name: "none".into(),
            unit: "count".into(),
            hints: vec![],
        };
        let def = parse_ontology_entry(&entry);
        assert!(def.category.is_none());
        assert!(def.hints.is_empty());
    }
}
