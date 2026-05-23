use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

pub type Score = f32;

// ── Coherence Ontology ──────────────────────────────────────────────
/// Rating system for logical patterns in the local vector database.
/// Isomorphic: applies identically to machine functions and human behaviours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CoherenceRating {
    /// Code compiled and executed with confirmed functional success.
    /// Human: task completed and produced the expected real-world effect.
    Success,
    /// Code compiles but does not produce the expected effect (Inoperoso).
    /// Human: task executed but no cognitive/material advancement detected.
    Inert,
    /// Compiler error or explicitly refuted logical pattern.
    /// Human: violated a self-imposed order (diet break, skipped training).
    Failure,
}

impl CoherenceRating {
    /// Numeric weight for scoring calculations.
    pub fn weight(self) -> Score {
        match self {
            Self::Success => 1.0,
            Self::Inert => 0.0,
            Self::Failure => -1.0,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Inert => "inert",
            Self::Failure => "failure",
        }
    }

    pub fn from_weight(w: Score) -> Self {
        if w >= 0.85 {
            Self::Success
        } else if w >= -0.15 {
            Self::Inert
        } else {
            Self::Failure
        }
    }
}

impl fmt::Display for CoherenceRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceNode {
    pub id: String,
    pub label: String,
    pub rating: CoherenceRating,
    pub axis_kind: AxisKind,
    pub domain: Option<String>,
    pub evidence: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub transition_count: u32,
}

impl CoherenceNode {
    pub fn new(label: &str, rating: CoherenceRating, axis_kind: AxisKind) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            label: label.to_string(),
            rating,
            axis_kind,
            domain: None,
            evidence: vec![],
            created_at: now,
            updated_at: now,
            transition_count: 0,
        }
    }

    /// Transition state: Success → Inert (function deemed inoperable by user).
    /// Only valid for Success → Inert downgrades. Failure is permanent.
    pub fn mark_inert(&mut self, reason: &str) -> bool {
        if self.rating != CoherenceRating::Success {
            return false;
        }
        self.rating = CoherenceRating::Inert;
        self.evidence.push(format!("INERT: {}", reason));
        self.updated_at = Utc::now();
        self.transition_count += 1;
        true
    }

    pub fn mark_failure(&mut self, reason: &str) {
        self.rating = CoherenceRating::Failure;
        self.evidence.push(format!("FAILURE: {}", reason));
        self.updated_at = Utc::now();
        self.transition_count += 1;
    }
}

/// Constructive refutation payload sent back to the phone interface
/// when a user categorization conflicts with established Success nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructiveRefutation {
    pub conflict_id: String,
    pub user_input_summary: String,
    pub conflicting_nodes: Vec<CoherenceNode>,
    pub suggestion: String,
    pub requires_pdca_recalibration: bool,
    pub timestamp: DateTime<Utc>,
}

impl ConstructiveRefutation {
    pub fn new(
        user_input: &str,
        conflicts: Vec<CoherenceNode>,
        suggestion: &str,
    ) -> Self {
        Self {
            conflict_id: Uuid::new_v4().to_string(),
            user_input_summary: user_input.to_string(),
            conflicting_nodes: conflicts,
            suggestion: suggestion.to_string(),
            requires_pdca_recalibration: true,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AxisKind {
    Human,
    Machine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanScoreAxis {
    Physical,
    Economic,
    Intellectual,
    Psychological,
}

impl HumanScoreAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Physical => "physical",
            Self::Economic => "economic",
            Self::Intellectual => "intellectual",
            Self::Psychological => "psychological",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineScoreAxis {
    Operational,
    Structural,
    Informational,
    Energetic,
}

impl MachineScoreAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Operational => "operational",
            Self::Structural => "structural",
            Self::Informational => "informational",
            Self::Energetic => "energetic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionDomain {
    Fabricate,
    Study,
    Construct,
    Bond,
    Heal,
}

impl ActionDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fabricate => "FABRICATE",
            Self::Study => "STUDY",
            Self::Construct => "CONSTRUCT",
            Self::Bond => "BOND",
            Self::Heal => "HEAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionMode {
    Lift,
    Rest,
    Create,
    Walk,
    Work,
    Learn,
}

impl ActionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lift => "LIFT",
            Self::Rest => "REST",
            Self::Create => "CREATE",
            Self::Walk => "WALK",
            Self::Work => "WORK",
            Self::Learn => "LEARN",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainDef {
    pub name: String,
    pub category: Option<String>,
    pub domain: ActionDomain,
    pub mode: ActionMode,
    pub axis_kind: AxisKind,
    pub axis_human: Option<HumanScoreAxis>,
    pub axis_machine: Option<MachineScoreAxis>,
    pub unit: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub domain_name: String,
    pub progress: Score,
    pub priority: Option<u32>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Goal {
    pub fn new(name: &str, domain_name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            domain_name: domain_name.to_string(),
            progress: 0.0,
            priority: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonalEntity {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub associations: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl PersonalEntity {
    pub fn new(name: &str, category: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            category: category.to_string(),
            description: String::new(),
            associations: vec![],
            metadata: HashMap::new(),
            first_seen: now,
            last_seen: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: String,
    pub goal_id: String,
    pub action: String,
    pub rationale: String,
    pub grade: Score,
    pub scores: HashMap<String, Score>,
    pub timestamp: DateTime<Utc>,
}

impl Experience {
    pub fn new(goal_id: &str, action: &str, grade: Score, rationale: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            goal_id: goal_id.to_string(),
            action: action.to_string(),
            rationale: rationale.to_string(),
            grade,
            scores: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DreamType {
    Mutation,
    Graft,
    Prune,
    CrossPollination,
}

impl DreamType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mutation => "MUTATION",
            Self::Graft => "GRAFT",
            Self::Prune => "PRUNE",
            Self::CrossPollination => "CROSS_POLLINATION",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dream {
    pub id: String,
    pub dream_type: DreamType,
    pub source_goal_id: String,
    pub source_path: Vec<String>,
    pub variation: Vec<String>,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub grade: Option<Score>,
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
    pub attributes: HashMap<String, String>,
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
    pub entities: HashMap<String, Entity>,
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
