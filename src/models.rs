use serde::{Deserialize, Serialize};

pub type Score = f32;

/// State of a goal in vector space. No human-readable qualia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub embedding: Vec<f32>,
    pub progress: Score,
}

impl Goal {
    pub fn new_vec(embedding: Vec<f32>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            embedding,
            progress: 0.0,
        }
    }
}

/// A dream is a generated vector from source vectors. No types, descriptions, or paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dream {
    pub id: String,
    pub source: Vec<f32>,
    pub embedding: Vec<f32>,
    pub grade: Option<Score>,
}

impl Dream {
    pub fn new(source: Vec<f32>, embedding: Vec<f32>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            embedding,
            grade: None,
        }
    }
}

/// Coherence is purely geometric — a node is just a vector with a density score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceNode {
    pub id: String,
    pub embedding: Vec<f32>,
    pub coherence_score: Score,
}

impl CoherenceNode {
    pub fn new(embedding: Vec<f32>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            embedding,
            coherence_score: 0.0,
        }
    }
}

/// An experience is a vector delta — before → after. No action text or rationale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: String,
    pub goal_id: String,
    pub before: Vec<f32>,
    pub after: Vec<f32>,
    pub delta: Vec<f32>,
    pub grade: Score,
}

impl Experience {
    pub fn new(goal_id: &str, before: Vec<f32>, after: Vec<f32>) -> Self {
        let delta: Vec<f32> = before.iter().zip(after.iter()).map(|(b, a)| a - b).collect();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            goal_id: goal_id.to_string(),
            before,
            after,
            delta,
            grade: 0.0,
        }
    }
}

/// Filtered context is a vector. No cleaned text, no vector_context string.
#[derive(Debug, Clone, Serialize)]
pub struct FilteredContext {
    pub embedding: Vec<f32>,
    pub valid: bool,
    pub token_estimate: usize,
}

/// Constructive refutation — kept for the consistency checker, but now vector-based.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructiveRefutation {
    pub conflict_id: String,
    pub query_embedding: Vec<f32>,
    pub conflicting_node_ids: Vec<String>,
    pub suggestion: String,
    pub coherence_gap: Score,
}

impl ConstructiveRefutation {
    pub fn new(query_embedding: Vec<f32>, conflicting_ids: Vec<String>, suggestion: &str, gap: Score) -> Self {
        Self {
            conflict_id: uuid::Uuid::new_v4().to_string(),
            query_embedding,
            conflicting_node_ids: conflicting_ids,
            suggestion: suggestion.to_string(),
            coherence_gap: gap,
        }
    }
}

/// CertifiedBranch — no label, no domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifiedBranch {
    pub branch_id: String,
    pub node_ids: Vec<String>,
    pub centroid: Vec<f32>,
    pub stability_score: Score,
}

/// IsolatedBranch — no label, no contradiction text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolatedBranch {
    pub branch_id: String,
    pub node_ids: Vec<String>,
    pub outlier_score: Score,
}

/// DreamResult — no scenario text, no collapse_chain text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamResult {
    pub dream_id: String,
    pub nodes_tested: Vec<String>,
    pub outcome: f32,
    pub prevented_failure: bool,
    pub coherence_delta: Score,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyEntry {
    pub name: String,
    pub category: Option<String>,
    pub domain: String,
    pub mode: String,
    pub axis_kind: String,
    pub axis_name: String,
    pub unit: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyConfig {
    pub kind: String,
    pub domains: Vec<OntologyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub description: Option<String>,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub predicate: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologicalMap {
    pub entities: std::collections::HashMap<String, Entity>,
    pub relationships: Vec<Relationship>,
}

impl OntologicalMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id.clone(), entity);
    }

    pub fn add_relationship(&mut self, rel: Relationship) {
        self.relationships.push(rel);
    }

    pub fn merge(&mut self, other: OntologicalMap) {
        for (id, entity) in other.entities {
            if let Some(existing) = self.entities.get_mut(&id) {
                for (k, v) in entity.attributes {
                    existing.attributes.insert(k, v);
                }
                if entity.description.is_some() {
                    existing.description = entity.description;
                }
            } else {
                self.entities.insert(id, entity);
            }
        }
        self.relationships.extend(other.relationships);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDef {
    pub name: String,
    pub category: Option<String>,
    pub unit: String,
    pub hints: Vec<String>,
}

// ── Helper: cosine similarity (used throughout) ───────────────────────

pub fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
    (dot / (na * nb)).clamp(-1.0, 1.0)
}

pub fn cosine_dist(a: &[f32], b: &[f32]) -> f32 {
    1.0 - cosine_sim(a, b)
}
