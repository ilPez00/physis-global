use crate::models::*;

pub const MACHINE_ONTOLOGY_NAME: &str = "machine_ontology";
pub const MACHINE_TYPE: &str = "machine";

pub fn resolve_nonhuman_domain<'a>(
    name: &str,
    domains: &'a std::collections::HashMap<String, DomainDef>,
) -> Option<&'a DomainDef> {
    let lower = name.to_lowercase();

    if let Some(def) = domains.get(name) {
        return Some(def);
    }

    for def in domains.values() {
        if def.hints.iter().any(|h| lower.contains(h)) {
            return Some(def);
        }
    }

    for def in domains.values() {
        if lower.contains(&def.name.to_lowercase()) {
            return Some(def);
        }
    }

    None
}

pub fn enrich_nonhuman_goal(
    goal: &Goal,
    domains: &std::collections::HashMap<String, DomainDef>,
) -> String {
    let def = resolve_nonhuman_domain(&goal.id, domains);
    match def {
        Some(d) => {
            format!(
                "• \"{}\" [{} +{}] progress={}%  [MACHINE]",
                goal.id,
                d.name,
                d.unit,
                (goal.progress * 100.0) as u32
            )
        }
        None => format!(
            "• \"{}\" [UNKNOWN_MACHINE_DOMAIN] progress={}%",
            goal.id,
            (goal.progress * 100.0) as u32
        ),
    }
}
