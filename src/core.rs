// ─── Physis Core Engine ──────────────────────────────────────────────
// The standalone runtime governing ontology, immunization, and token
// economy across both Aura (local) and Praxis (global) contexts.
//
// Three architectural domains:
//  1. UBERWIKI — Immutable geometric causal structures (±1.0 weights)
//  2. FUNZIONE COMPRESSIONE (Waking Phase) — Real-time glove stripping
//     raw logs down to minimal causal tokens before IA Host ingestion.
//  3. FUNZIONE SOGNO (Dream Phase) — Background async processing of
//     failure fragments against universal myths; generates Effective
//     Actions and recalibrates the Coherence Index.
//
// The Compression function operates as a filter during AURA/PRAXIS
// operations; the Dream function collides contingent failures with
// the UberWiki's universal myths to find the missing causal link.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::OntologyLoader;
use crate::models::*;
use crate::trie::DynamicVectorTrie;

// ── Core Filter Struct ─────────────────────────────────────────────

/// The Physis Core engine: wraps every input through a causal filter that
/// strips syntactic noise and retains only essential constraints + vector history.
///
/// Governs the three architectural domains:
///  - UBERWIKI (the `wiki` trie): immutable geometric causal structures
///  - FUNZIONE COMPRESSIONE (`filtra_contesto`, `compress_logs`): real-time input glove
///  - FUNZIONE SOGNO (`dream`, `certify_branches`): background async processing
#[derive(Debug)]
pub struct PhysisCore {
    /// All registered coherence nodes (machine + human, unified pool).
    pub nodes: HashMap<String, CoherenceNode>,
    /// Indices for fast lookup: label → node_id.
    label_index: HashMap<String, String>,
    /// Domain → node_ids index.
    domain_index: HashMap<String, Vec<String>>,
    /// The local vector trie (Uber Wiki locale).
    pub wiki: DynamicVectorTrie,
    /// Certified branches ready for global-wiki merge.
    pub certified_branches: Vec<CertifiedBranch>,
    /// Isolated branches (contradicted, kept for forensics).
    pub isolated_branches: Vec<IsolatedBranch>,
    /// Dream simulation results.
    pub dream_archive: Vec<DreamResult>,
    /// Noise-stripping patterns (compiled once for performance).
    noise_patterns: Vec<Regex>,
    /// Causal-connector tokens preserved during filtering.
    causal_connectors: HashSet<&'static str>,
    /// Sub-sentence delimiters for Wenyan-style semantic compression.
    compression_atoms: Vec<CompressionAtom>,
}

/// Output of the context filter — stripped, validated, with injected vector history.
#[derive(Debug, Clone)]
pub struct FilteredContext {
    /// The cleaned input, reduced to essential causal constraints.
    pub cleaned: String,
    /// Injected vector-context from the local wiki.
    pub vector_context: String,
    /// Any consistency conflict detected.
    pub conflict: Option<ConstructiveRefutation>,
    /// Whether the context passed validation.
    pub valid: bool,
    /// Token count estimate of the cleaned output.
    pub token_estimate: usize,
}

/// Result of a consistency check against existing nodes.
#[derive(Debug, Clone)]
pub enum ConsistencyResult {
    /// Clean: no conflicts with established Success/geometric nodes.
    Clean,
    /// Conflict found: execution suspended; refutation payload generated.
    Conflict(ConstructiveRefutation),
}

/// A certified branch — stable, coherent, ready for global-wiki merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifiedBranch {
    pub branch_id: String,
    pub label: String,
    pub node_ids: Vec<String>,
    pub stability_score: Score,
    pub certified_at: DateTime<Utc>,
    pub domain: Option<String>,
}

/// An isolated branch — contradicted, kept for forensics analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolatedBranch {
    pub branch_id: String,
    pub label: String,
    pub node_ids: Vec<String>,
    pub contradiction: String,
    pub isolated_at: DateTime<Utc>,
}

/// Result of a predictive dream simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamResult {
    pub dream_id: String,
    pub scenario: String,
    pub nodes_tested: Vec<String>,
    pub outcome: DreamOutcome,
    pub collapse_chain: Vec<String>,
    pub prevented_failure: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DreamOutcome {
    /// Scenario is stable; branch survives stress test.
    Stable,
    /// Logical collapse detected; node downgraded preventively.
    Collapsed,
    /// Inconclusive — needs more data.
    Inconclusive,
}

/// A compression atom: semantic unit for Wenyan-density serialization.
#[derive(Debug, Clone)]
struct CompressionAtom {
    token: String,
    weight: Score,
    category: AtomCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AtomCategory {
    Causal,    // if, then, because, therefore
    Entity,    // named entities, goals, domains
    Metric,    // numbers, scores, counts
    Action,    // verbs, PDCA actions
    Temporal,  // time references
    Filler,    // discardable
}

// ── Implementation ─────────────────────────────────────────────────

impl PhysisCore {
    /// Create a new empty filter.
    pub fn new() -> Self {
        let causal_connectors: HashSet<&'static str> = [
            "perché", "quindi", "se", "allora", "ma", "poiché", "affinché",
            "nonostante", "dunque", "perciò", "benché", "sebbene",
            "because", "therefore", "if", "then", "but", "since", "so",
            "due_to", "caused_by", "leads_to", "prevents", "enables",
            "non", "not", "no", "never", "mai", "nessuno", "niente",
            "senza", "without", "neither", "nor",
        ]
        .into();

        let noise_patterns = vec![
            Regex::new(r"(?i)\b(um|uh|eh|ah|mmh|beh|cioè|tipo|praticamente|in pratica|diciamo|ecco)\b").unwrap(),
            Regex::new(r"\s{2,}").unwrap(),
            Regex::new(r"^\s*[-•·▪▸]\s*").unwrap(),
            Regex::new(r"\b(più o meno|all'incirca|circa|quasi)\b").unwrap(),
        ];

        Self {
            nodes: HashMap::new(),
            label_index: HashMap::new(),
            domain_index: HashMap::new(),
            wiki: DynamicVectorTrie::new(),
            certified_branches: vec![],
            isolated_branches: vec![],
            dream_archive: vec![],
            noise_patterns,
            causal_connectors,
            compression_atoms: vec![],
        }
    }

    /// Create with an existing wiki trie (loads from disk or merges).
    pub fn with_wiki(wiki: DynamicVectorTrie) -> Self {
        let mut g = Self::new();
        g.wiki = wiki;
        g
    }

    // ── Node Management ──────────────────────────────────────────

    /// Register a new coherence node. Returns the node id.
    pub fn register_node(
        &mut self,
        label: &str,
        rating: CoherenceRating,
        axis_kind: AxisKind,
        domain: Option<String>,
    ) -> String {
        let mut node = CoherenceNode::new(label, rating, axis_kind);
        node.domain = domain.clone();
        let id = node.id.clone();

        self.label_index.insert(label.to_lowercase(), id.clone());
        if let Some(ref dom) = domain {
            self.domain_index
                .entry(dom.clone())
                .or_default()
                .push(id.clone());
        }
        self.nodes.insert(id.clone(), node);
        id
    }

    /// Lookup a node by label (case-insensitive).
    pub fn find_by_label(&self, label: &str) -> Option<&CoherenceNode> {
        self.label_index
            .get(&label.to_lowercase())
            .and_then(|id| self.nodes.get(id))
    }

    /// Transition a node: Success → Inert (user reports function is inoperable).
    /// This is the key isomorphism: same downgrade path for code and human tasks.
    pub fn mark_inert(&mut self, label: &str, reason: &str) -> bool {
        if let Some(id) = self.label_index.get(&label.to_lowercase()).cloned() {
            if let Some(node) = self.nodes.get_mut(&id) {
                return node.mark_inert(reason);
            }
        }
        false
    }

    /// Mark a node as Failure (violation of established constraint).
    pub fn mark_failure(&mut self, label: &str, reason: &str) -> bool {
        if let Some(id) = self.label_index.get(&label.to_lowercase()).cloned() {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.mark_failure(reason);
                return true;
            }
        }
        false
    }

    // ── THE CORE: filtra_contesto ────────────────────────────────

    /// Main entry point: filter raw input through the Rachmaninov Glove.
    ///
    /// 1. Scrubs syntactic noise (filler words, hedging, redundant tokens).
    /// 2. Extracts causal constraints and injects relevant vector history.
    /// 3. Checks consistency against established Success/geometric nodes.
    /// 4. If conflict found, suspends and generates a ConstructiveRefutation.
    ///
    /// The `axis_kind` parameter defines whether we're filtering a machine or
    /// human input — same algorithm, different ontology index.
    pub fn filtra_contesto(
        &self,
        input_grezzo: &str,
        axis_kind: AxisKind,
        _ontology: &OntologyLoader,
    ) -> FilteredContext {
        // ── Phase 1: scrub syntactic noise ───────────────────────
        let mut cleaned = input_grezzo.to_string();
        for pat in &self.noise_patterns {
            cleaned = pat.replace_all(&cleaned, "").to_string();
        }
        // Collapse whitespace
        cleaned = Regex::new(r"\s+")
            .map(|re| re.replace_all(&cleaned.trim(), " ").to_string())
            .unwrap_or(cleaned);

        // ── Phase 2: extract causal constraints ──────────────────
        let causal_tokens: Vec<&str> = cleaned
            .split_whitespace()
            .filter(|w| {
                let lw = w.to_lowercase();
                // Preserve causal connectors, entities, metrics
                self.causal_connectors
                    .iter()
                    .any(|c| lw.contains(c))
                    || self.label_index.contains_key(&lw)
                    || lw.chars().any(|c| c.is_numeric())
                    || lw.starts_with('@')  // entity references
                    || lw.starts_with('#')  // tag references
            })
            .collect();

        let causal_core = causal_tokens.join(" ");
        let token_estimate = causal_core.split_whitespace().count();

        // ── Phase 3: inject vector history ───────────────────────
        let search_terms: Vec<&str> = causal_tokens
            .iter()
            .take(8)
            .copied()
            .collect();
        let vector_context = self.wiki.prefix_search_flat(&search_terms, 200);

        // ── Phase 4: consistency check ──────────────────────────
        let conflict = match self.check_consistency(&causal_core, axis_kind) {
            ConsistencyResult::Clean => None,
            ConsistencyResult::Conflict(refutation) => Some(refutation),
        };

        let valid = conflict.is_none();

        FilteredContext {
            cleaned: causal_core,
            vector_context,
            conflict,
            valid,
            token_estimate,
        }
    }

    /// Check if a new categorization conflicts with established Success
    /// nodes or geometric laws already consolidated in the local wiki.
    pub fn check_consistency(&self, input: &str, axis_kind: AxisKind) -> ConsistencyResult {
        let input_lower = input.to_lowercase();

        // Collect nodes that could conflict: Success nodes in the same axis
        let potential_conflicts: Vec<&CoherenceNode> = self
            .nodes
            .values()
            .filter(|n| n.axis_kind == axis_kind && n.rating == CoherenceRating::Success)
            .filter(|n| {
                // Check for semantic negation: if user says "X is not Y" and
                // we have a Success node asserting "X is Y", that's a conflict.
                let label_lower = n.label.to_lowercase();
                // Simple heuristic: overlap in key terms but contradictory polarity
                has_semantic_overlap(&input_lower, &label_lower)
                    && contains_negation(&input_lower)
            })
            .collect();

        if potential_conflicts.is_empty() {
            return ConsistencyResult::Clean;
        }

        // Build constructive refutation
        let suggestion = format!(
            "Il sistema rileva {} nodi affermativi in conflitto con questa categorizzazione. \
             Ricalibrare il ciclo PDCA: verificare le evidenze contrarie prima di procedere.",
            potential_conflicts.len()
        );

        let conflicts_cloned: Vec<CoherenceNode> =
            potential_conflicts.into_iter().cloned().collect();

        ConsistencyResult::Conflict(ConstructiveRefutation::new(
            input,
            conflicts_cloned,
            &suggestion,
        ))
    }

    // ── Branch Certification ─────────────────────────────────────

    /// Certify stable branches. A branch is a cluster of nodes in one domain.
    /// A branch is certifiable when all its nodes are Success and have been
    /// stable (no transitions) for at least `min_stability_epochs`.
    pub fn certify_branches(&mut self, _ontology: &OntologyLoader) -> Vec<CertifiedBranch> {
        let mut newly_certified = Vec::new();

        for (domain, node_ids) in &self.domain_index {
            // Skip already-certified branches.
            if self.certified_branches.iter().any(|b| b.domain.as_deref() == Some(domain.as_str())) {
                continue;
            }

            let domain_nodes: Vec<&CoherenceNode> = node_ids
                .iter()
                .filter_map(|id| self.nodes.get(id))
                .collect();

            if domain_nodes.is_empty() {
                continue;
            }

            // A branch is certifiable if ALL nodes are Success with 0 transitions.
            let all_success = domain_nodes
                .iter()
                .all(|n| n.rating == CoherenceRating::Success && n.transition_count == 0);

            if !all_success {
                continue;
            }

            let stability_score = domain_nodes.len() as Score;

            let branch = CertifiedBranch {
                branch_id: Uuid::new_v4().to_string(),
                label: domain.clone(),
                node_ids: node_ids.clone(),
                stability_score,
                certified_at: Utc::now(),
                domain: Some(domain.clone()),
            };

            newly_certified.push(branch.clone());
            self.certified_branches.push(branch);
        }

        newly_certified
    }

    /// Detect and isolate contradictory branches.
    /// A contradiction exists when two nodes in the same domain have opposing ratings
    /// (one Success, one Failure) with overlapping labels.
    pub fn detect_contradictions(&mut self) -> Vec<IsolatedBranch> {
        let mut isolated = Vec::new();

        for (domain, node_ids) in &self.domain_index {
            let domain_nodes: Vec<&CoherenceNode> = node_ids
                .iter()
                .filter_map(|id| self.nodes.get(id))
                .collect();

            let has_success = domain_nodes.iter().any(|n| n.rating == CoherenceRating::Success);
            let has_failure = domain_nodes.iter().any(|n| n.rating == CoherenceRating::Failure);

            if has_success && has_failure {
                let failure_labels: Vec<String> = domain_nodes
                    .iter()
                    .filter(|n| n.rating == CoherenceRating::Failure)
                    .map(|n| n.label.clone())
                    .collect();

                let branch = IsolatedBranch {
                    branch_id: Uuid::new_v4().to_string(),
                    label: format!("{}_CONTRADICTED", domain),
                    node_ids: node_ids.clone(),
                    contradiction: format!(
                        "Success/Failure clash in domain '{}': failing nodes: [{}]",
                        domain,
                        failure_labels.join(", ")
                    ),
                    isolated_at: Utc::now(),
                };

                isolated.push(branch.clone());
                self.isolated_branches.push(branch);
            }
        }

        isolated
    }

    // ── Compression: Wenyan-density semantic summarisation ───────

    /// Compress daily logs into pure causal rules during idle/sleep.
    /// Applies Classical-Chinese/Wenyan-style density: strip temporaries,
    /// retain only subject-predicate-consequence chains.
    pub fn compress_logs(&self, raw_logs: &[String]) -> String {
        let mut rules = Vec::new();
        let mut atom_cache: HashMap<String, CompressionAtom> = HashMap::new();

        for log in raw_logs {
            let atoms = self.decompose_into_atoms(log, &mut atom_cache);
            let rule = self.atoms_to_causal_rule(&atoms);
            if !rule.is_empty() {
                rules.push(rule);
            }
        }

        // Deduplicate and sort by weight
        let mut seen = HashSet::new();
        rules.retain(|r| seen.insert(r.clone()));
        rules.truncate(200); // max compressed rules per cycle

        // Serialize in dense format: RULE│RUL E│RULE
        rules.join("│")
    }

    /// Decompose a raw text into compression atoms.
    fn decompose_into_atoms(
        &self,
        text: &str,
        cache: &mut HashMap<String, CompressionAtom>,
    ) -> Vec<CompressionAtom> {
        let mut atoms = Vec::new();

        for word in text.split_whitespace() {
            let key = word.to_lowercase();
            let atom = cache.entry(key.clone()).or_insert_with(|| {
                let (category, weight) = classify_token(word);
                CompressionAtom {
                    token: word.to_string(),
                    weight,
                    category,
                }
            });

            if atom.category != AtomCategory::Filler {
                atoms.push(CompressionAtom {
                    token: atom.token.clone(),
                    weight: atom.weight,
                    category: atom.category,
                });
            }
        }



        atoms
    }

    /// Convert compression atoms into a dense causal rule string.
    fn atoms_to_causal_rule(&self, atoms: &[CompressionAtom]) -> String {
        if atoms.is_empty() {
            return String::new();
        }
        let significant: Vec<&CompressionAtom> = atoms
            .iter()
            .filter(|a| a.category != AtomCategory::Filler)
            .collect();
        if significant.is_empty() {
            return String::new();
        }
        significant
            .iter()
            .map(|a| a.token.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    // ── Dream: Predictive Collapse Simulation ───────────────

    fn simulate_collapse(&self, node: &CoherenceNode) -> Vec<String> {
        let mut chain = vec![node.label.clone()];
        for other in self.nodes.values() {
            if other.id != node.id
                && other.label.to_lowercase().contains(&node.label.to_lowercase())
            {
                chain.push(other.label.clone());
            }
        }
        chain
    }
}

// ── Helper Functions ──────────────────────────────────────────

fn has_semantic_overlap(a: &str, b: &str) -> bool {
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();
    words_a.iter().any(|w| w.len() > 2 && words_b.contains(w))
}

fn contains_negation(text: &str) -> bool {
    let negators = [
        "non", "no", "not", "mai", "never", "nessuno", "niente",
        "ne", "neanche", "neppure", "senza", "without",
    ];
    negators.iter().any(|n| {
        let lower = text.to_lowercase();
        lower == *n
            || lower.starts_with(&format!("{} ", n))
            || lower.ends_with(&format!(" {}", n))
            || lower.starts_with(&format!("{}-", n))
            || lower.contains(&format!(" {} ", n))
    })
}

fn classify_token(token: &str) -> (AtomCategory, Score) {
    let lower = token.to_lowercase();

    if lower.parse::<f64>().is_ok() {
        return (AtomCategory::Metric, 0.8);
    }
    if lower.chars().any(|c| c.is_numeric()) && lower.len() <= 6 {
        return (AtomCategory::Metric, 0.7);
    }

    let causals: &[&str] = &[
        "perche", "quindi", "se", "allora", "ma", "poiche", "affinche",
        "dunque", "percio", "benche", "because", "therefore", "if", "then",
        "but", "since", "so", "due", "to", "caused", "by",
    ];
    if causals.iter().any(|c| lower == *c) {
        return (AtomCategory::Causal, 0.9);
    }

    let temporals: &[&str] = &[
        "oggi", "ieri", "domani", "ora", "minuti", "ore", "giorni",
        "settimana", "mese", "anno", "today", "yesterday", "tomorrow",
    ];
    if temporals.iter().any(|t| lower.contains(t)) {
        return (AtomCategory::Temporal, 0.5);
    }

    if token.starts_with('@')
        || token.starts_with('#')
        || (token.len() > 3 && token.chars().next().map_or(false, |c| c.is_uppercase()))
    {
        return (AtomCategory::Entity, 0.7);
    }

    if lower.ends_with("are")
        || lower.ends_with("ere")
        || lower.ends_with("ire")
        || lower.ends_with("ing")
        || lower.ends_with("ed")
    {
        return (AtomCategory::Action, 0.6);
    }

    (AtomCategory::Filler, 0.1)
}

impl PhysisCore {
    // ── Dream Phase ─────────────────────────────────────────────────

    /// Execute predictive dream simulation on Inert (0.0) or uncertified nodes.
    /// Collides contingent failures with universal myths from the local UberWiki
    /// to find missing causal links and prevent real-world failures.
    pub fn dream(&mut self, _ontology: &OntologyLoader) -> Vec<DreamResult> {
        let inert_nodes: Vec<CoherenceNode> = self
            .nodes
            .values()
            .filter(|n| n.rating == CoherenceRating::Inert || n.transition_count > 0)
            .cloned()
            .collect();

        let mut results = Vec::new();

        for node in &inert_nodes {
            if self
                .dream_archive
                .iter()
                .any(|d| d.nodes_tested.contains(&node.id))
            {
                continue;
            }

            let scenario = format!("Constraint collapse: '{}' fails under stress", node.label);

            let collapse_chain = self.simulate_collapse(node);

            let outcome = if collapse_chain.len() > 1 {
                self.mark_failure(&node.label, "dream simulation: collapse chain detected");
                DreamOutcome::Collapsed
            } else {
                DreamOutcome::Stable
            };

            let result = DreamResult {
                dream_id: Uuid::new_v4().to_string(),
                scenario,
                nodes_tested: vec![node.id.clone()],
                outcome,
                collapse_chain,
                prevented_failure: outcome == DreamOutcome::Collapsed,
                timestamp: Utc::now(),
            };

            results.push(result.clone());
            self.dream_archive.push(result);
        }

        results
    }

    // ── Coherence Index ──────────────────────────────────────────

    /// Compute the global coherence index (stock-market-like metric).
    /// Averages the weights of nodes, optionally filtered by AxisKind.
    /// Returns 1.0 for an empty set (no data = no friction).
    pub fn coherence_index(&self, axis_filter: Option<AxisKind>) -> Score {
        let relevant: Vec<Score> = self
            .nodes
            .values()
            .filter(|n| axis_filter.map_or(true, |ax| n.axis_kind == ax))
            .map(|n| n.rating.weight())
            .collect();

        if relevant.is_empty() {
            return 1.0;
        }

        let sum: Score = relevant.iter().sum();
        sum / (relevant.len() as Score)
    }
}

impl Default for PhysisCore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Snapshots and Behavioural ─────────────────────────────────

impl PhysisCore {
    pub fn snapshot(&self) -> CoherenceSnapshot {
        CoherenceSnapshot {
            total_nodes: self.nodes.len(),
            success_count: self.nodes.values().filter(|n| n.rating == CoherenceRating::Success).count(),
            inert_count: self.nodes.values().filter(|n| n.rating == CoherenceRating::Inert).count(),
            failure_count: self.nodes.values().filter(|n| n.rating == CoherenceRating::Failure).count(),
            certified_branches_count: self.certified_branches.len(),
            isolated_branches_count: self.isolated_branches.len(),
            dream_cycle_count: self.dream_archive.len(),
            coherence_index: self.coherence_index(None),
        }
    }

    pub fn register_behavioural_vector(
        &mut self,
        domain: &str,
        action: &str,
        rating: CoherenceRating,
        reason: &str,
    ) -> String {
        let label = format!("{}:{}", domain.to_lowercase().replace(' ', "_"), action);
        let id = self.register_node(&label, rating, AxisKind::Human, Some(domain.into()));
        let wiki_entry = format!(
            "{} {} {} {}",
            label,
            rating.weight(),
            reason,
            chrono::Utc::now().format("%Y-%m-%d")
        );
        self.wiki.insert_str(&wiki_entry);
        id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceSnapshot {
    pub total_nodes: usize,
    pub success_count: usize,
    pub inert_count: usize,
    pub failure_count: usize,
    pub certified_branches_count: usize,
    pub isolated_branches_count: usize,
    pub dream_cycle_count: usize,
    pub coherence_index: Score,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_core() -> PhysisCore {
        let mut g = PhysisCore::new();
        g.register_node("exercise:running", CoherenceRating::Success, AxisKind::Human, Some("Body & Fitness".into()));
        g.register_node("diet:no_sugar", CoherenceRating::Success, AxisKind::Human, Some("Body & Fitness".into()));
        g.register_node("compile:physis_core", CoherenceRating::Success, AxisKind::Machine, Some("software".into()));
        g.wiki.insert_str("exercise running success D0");
        g.wiki.insert_str("diet no_sugar success D0");
        g.wiki.insert_str("compile physis_core success D0");
        g
    }

    fn fixture_ontology() -> OntologyLoader {
        OntologyLoader::new()
    }

    #[test]
    fn test_filtra_contesto_clean_input() {
        let g = fixture_core();
        let onto = fixture_ontology();
        let result = g.filtra_contesto("new exercise:swimming completed successfully", AxisKind::Human, &onto);
        assert!(result.valid);
        assert!(result.conflict.is_none());
        assert!(!result.cleaned.is_empty());
    }

    #[test]
    fn test_filtra_contesto_detects_contradiction() {
        let g = fixture_core();
        let onto = fixture_ontology();
        let result = g.filtra_contesto("exercise:running is not producing any effect", AxisKind::Human, &onto);
        assert!(!result.valid);
        assert!(result.conflict.is_some());
    }

    #[test]
    fn test_certify_branches() {
        let mut g = fixture_core();
        let onto = fixture_ontology();
        let certified = g.certify_branches(&onto);
        assert!(certified.iter().any(|b| b.domain.as_deref() == Some("Body & Fitness")));
    }

    #[test]
    fn test_detect_contradictions() {
        let mut g = fixture_core();
        g.register_node("exercise:yoga", CoherenceRating::Failure, AxisKind::Human, Some("Body & Fitness".into()));
        let contradictions = g.detect_contradictions();
        assert!(!contradictions.is_empty());
    }

    #[test]
    fn test_dream_on_inert_nodes() {
        let mut g = fixture_core();
        g.mark_inert("exercise:running", "no endurance gain detected");
        let onto = fixture_ontology();
        let dreams = g.dream(&onto);
        assert!(!dreams.is_empty());
    }

    #[test]
    fn test_coherence_index() {
        let mut g = fixture_core();
        assert!((g.coherence_index(Some(AxisKind::Human)) - 1.0).abs() < 0.01);
        g.register_node("diet:ate_cake", CoherenceRating::Failure, AxisKind::Human, Some("Body & Fitness".into()));
        let idx = g.coherence_index(Some(AxisKind::Human));
        assert!(idx < 1.0);
        assert!(idx > 0.0);
    }

    #[test]
    fn test_compress_logs() {
        let g = fixture_core();
        let logs = vec![
            "I went for a run today and I felt very good about it".to_string(),
            "Today I studied for three hours but I did not really understand the material well".to_string(),
            "I avoided sugar today because I want to stay healthy".to_string(),
        ];
        let compressed = g.compress_logs(&logs);
        assert!(!compressed.is_empty());
        let input_len: usize = logs.iter().map(|l| l.len()).sum();
        assert!(compressed.len() < input_len, "compression should reduce size");
    }

    #[test]
    fn test_snapshot() {
        let g = fixture_core();
        let snap = g.snapshot();
        assert_eq!(snap.total_nodes, 3);
        assert_eq!(snap.success_count, 3);
    }

    #[test]
    fn test_register_behavioural_vector() {
        let mut g = fixture_core();
        let id = g.register_behavioural_vector("Body & Fitness", "morning_yoga", CoherenceRating::Success, "completed routine");
        assert!(!id.is_empty());
        assert!(g.nodes.contains_key(&id));
    }
}
