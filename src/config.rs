use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::models::*;
use crate::ontology::HUMAN_ONTOLOGY_NAME;
use crate::ontology_nonhuman::MACHINE_ONTOLOGY_NAME;

static BUILTIN_HUMAN_JSON: &str = include_str!("../config/praxis_ontology.json");
static BUILTIN_MACHINE_JSON: &str = include_str!("../config/machine_ontology.json");
static DEFAULT_PHYSIS_DIR: &str = ".physis";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysisConfig {
    pub data_dir: PathBuf,
    pub ontologies: Vec<OntologySource>,
    pub network_scan_interval_secs: u64,
    pub dream_batch_size: usize,
    pub pdca_stagnant_threshold: f32,
    pub pdca_stagnant_window: usize,
    pub watch_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologySource {
    pub name: String,
    pub path: Option<PathBuf>,
    pub kind: String,
    pub enabled: bool,
}

impl Default for PhysisConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            data_dir: PathBuf::from(home).join(DEFAULT_PHYSIS_DIR),
            ontologies: vec![
                OntologySource {
                    name: HUMAN_ONTOLOGY_NAME.to_string(),
                    path: None,
                    kind: "human".to_string(),
                    enabled: true,
                },
                OntologySource {
                    name: MACHINE_ONTOLOGY_NAME.to_string(),
                    path: None,
                    kind: "machine".to_string(),
                    enabled: true,
                },
            ],
            network_scan_interval_secs: 60,
            dream_batch_size: 5,
            pdca_stagnant_threshold: 0.2,
            pdca_stagnant_window: 5,
            watch_dirs: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct OntologyLoader {
    pub human_domains: HashMap<String, DomainDef>,
    pub machine_domains: HashMap<String, DomainDef>,
    pub custom_domains: HashMap<String, DomainDef>,
}

impl OntologyLoader {
    pub fn new() -> Self {
        Self {
            human_domains: HashMap::new(),
            machine_domains: HashMap::new(),
            custom_domains: HashMap::new(),
        }
    }

    pub fn load_builtin_human() -> HashMap<String, DomainDef> {
        let entries: OntologyConfig = serde_json::from_str(BUILTIN_HUMAN_JSON)
            .expect("Built-in human ontology is valid JSON");
        Self::entries_to_map(&entries.domains)
    }

    pub fn load_builtin_machine() -> HashMap<String, DomainDef> {
        let entries: OntologyConfig = serde_json::from_str(BUILTIN_MACHINE_JSON)
            .expect("Built-in machine ontology is valid JSON");
        Self::entries_to_map(&entries.domains)
    }

    pub fn load_from_path(path: &Path) -> anyhow::Result<HashMap<String, DomainDef>> {
        let contents = std::fs::read_to_string(path)?;
        let config: OntologyConfig = serde_json::from_str(&contents)?;
        Ok(Self::entries_to_map(&config.domains))
    }

    pub fn load_from_str(json: &str) -> anyhow::Result<HashMap<String, DomainDef>> {
        let config: OntologyConfig = serde_json::from_str(json)?;
        Ok(Self::entries_to_map(&config.domains))
    }

    fn entries_to_map(entries: &[OntologyEntry]) -> HashMap<String, DomainDef> {
        let mut map = HashMap::new();
        for e in entries {
            map.insert(
                e.name.clone(),
                DomainDef {
                    name: e.name.clone(),
                    category: e.category.clone(),
                    unit: e.unit.clone(),
                    hints: e.hints.clone(),
                },
            );
        }
        map
    }

    pub fn load_all(config: &PhysisConfig) -> Self {
        let mut loader = Self::new();

        for source in &config.ontologies {
            if !source.enabled {
                continue;
            }
            let map = if let Some(ref path) = source.path {
                if path.exists() {
                    Self::load_from_path(path).unwrap_or_else(|e| {
                        eprintln!("Warning: failed to load ontology '{}' from {}: {}", source.name, path.display(), e);
                        HashMap::new()
                    })
                } else {
                    eprintln!("Warning: ontology path {} not found", path.display());
                    HashMap::new()
                }
            } else if source.name == HUMAN_ONTOLOGY_NAME {
                Self::load_builtin_human()
            } else if source.name == MACHINE_ONTOLOGY_NAME {
                Self::load_builtin_machine()
            } else {
                HashMap::new()
            };

            for (name, def) in map {
                match source.kind.as_str() {
                    "human" => { loader.human_domains.insert(name, def); }
                    "machine" => { loader.machine_domains.insert(name, def); }
                    _ => { loader.custom_domains.insert(name, def); }
                }
            }
        }

        loader
    }

    pub fn resolve_domain(&self, _goal_name: &str) -> Option<&DomainDef> {
        self.human_domains.values().next()
    }

    pub fn enrich_goal(&self, goal: &Goal) -> String {
        let _ = goal;
        let def = self.human_domains.values().next();
        match def {
            Some(d) => format!("[{}] progress={}%", d.unit, (0.0 * 100.0) as u32),
            None => format!("[VECTOR] progress={}%", (0.0 * 100.0) as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_human_ontology_count() {
        let map = OntologyLoader::load_builtin_human();
        assert!(map.len() >= 14, "expected at least 14 human domains, got {}", map.len());
    }

    #[test]
    fn test_builtin_machine_ontology_count() {
        let map = OntologyLoader::load_builtin_machine();
        assert!(map.len() >= 50, "expected at least 50 machine domains, got {}", map.len());
    }

    #[test]
    fn test_domain_resolution_exact() {
        let map = OntologyLoader::load_builtin_human();
        let def = map.get("Body & Fitness");
        assert!(def.is_some(), "Body & Fitness should be found");
        if let Some(d) = def {
            assert!(d.hints.iter().any(|h| h == "exercise"), "expected 'exercise' hint");
        }
    }

    #[test]
    fn test_load_from_str_valid() {
        let json = r#"{"kind":"human","domains":[{"name":"Test","category":null,"domain":"WORK","mode":"WORK","axis_kind":"human","axis_name":"physical","unit":"units","hints":["test"]}]}"#;
        let map = OntologyLoader::load_from_str(json).unwrap();
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_load_from_str_invalid() {
        let result = OntologyLoader::load_from_str("not valid json");
        assert!(result.is_err());
    }
}
