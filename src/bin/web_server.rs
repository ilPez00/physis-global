use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use physis::actor::PDCActor;
use physis::config::{OntologyLoader, PhysisConfig};
use physis::core::PhysisCore;
use physis::dream::DreamEngine;
use physis::embed::{RandomProjectionEmbedder, VectorEmbed};
use physis::mapper::OntologyMapper;
use physis::models::{Goal, Score};
use physis::CoherenceSnapshot;

struct AppState {
    core: PhysisCore,
    mapper: OntologyMapper,
    actor: PDCActor,
    dreams: DreamEngine,
    ontology: OntologyLoader,
    goals: Vec<Goal>,
    embedder: RandomProjectionEmbedder,
}

type SharedState = Arc<Mutex<AppState>>;

#[derive(Deserialize)]
struct QueryParams {
    q: String,
    #[serde(default = "default_max")]
    max: usize,
}
fn default_max() -> usize { 10 }

#[derive(Deserialize)]
struct ScanRequest {
    dir: String,
}

#[derive(Deserialize)]
struct DreamGenerateRequest {
    #[serde(default = "default_dream_count")]
    count: usize,
    #[serde(default)]
    force: bool,
}
fn default_dream_count() -> usize { 5 }

#[derive(Deserialize)]
struct DreamEvaluateRequest {
    id: String,
    grade: Score,
}

#[derive(Deserialize)]
struct CoherenceRegisterRequest {
    input: String,
}

#[derive(Deserialize)]
struct ContextFilterRequest {
    input: String,
}

#[derive(Deserialize)]
struct CompressLogsRequest {
    logs: Vec<String>,
}

#[derive(Deserialize)]
struct PDCARequest {
    goal_id: String,
    state_vector: Vec<f32>,
}

#[derive(Serialize)]
struct StatsResponse {
    mapper: HashMap<String, usize>,
    core: CoherenceSnapshot,
}

#[derive(Serialize)]
struct ScanResponse {
    goals: Vec<Goal>,
    count: usize,
}

#[derive(Serialize)]
struct QueryResponse {
    results: Vec<Vec<String>>,
    count: usize,
}

#[derive(Serialize)]
struct CoherenceRegisterResponse {
    node_id: String,
}

#[derive(Serialize)]
struct FilterResponse {
    embedding: Vec<f32>,
    valid: bool,
    token_estimate: usize,
}

#[derive(Serialize)]
struct CompressResponse {
    compressed: String,
    input_count: usize,
    output_chars: usize,
}

#[derive(Serialize)]
struct IngestResponse {
    goal: Goal,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn err_response(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (status, Json(ErrorResponse { error: msg.to_string() }))
}

// ── Handlers ──────────────────────────────────────────────────────────

async fn health() -> Json<&'static str> {
    Json("ok")
}

async fn stats_handler(State(state): State<SharedState>) -> Json<StatsResponse> {
    let s = state.lock().unwrap();
    Json(StatsResponse {
        mapper: s.mapper.stats(),
        core: s.core.snapshot(),
    })
}

async fn query_handler(
    State(state): State<SharedState>,
    Query(params): Query<QueryParams>,
) -> Json<QueryResponse> {
    let s = state.lock().unwrap();
    let results = s.mapper.query(&params.q);
    let limited: Vec<Vec<String>> = results.into_iter().take(params.max).collect();
    Json(QueryResponse { count: limited.len(), results: limited })
}

async fn scan_handler(
    State(state): State<SharedState>,
    Json(req): Json<ScanRequest>,
) -> Result<Json<ScanResponse>, (StatusCode, Json<ErrorResponse>)> {
    let path = PathBuf::from(&req.dir);
    if !path.exists() {
        return Err(err_response(StatusCode::BAD_REQUEST, &format!("directory not found: {}", req.dir)));
    }
    let mut s = state.lock().unwrap();
    let goals = s.mapper.map_filesystem(&path, None);
    s.goals = goals.clone();
    Ok(Json(ScanResponse { count: goals.len(), goals }))
}

async fn dream_generate_handler(
    State(state): State<SharedState>,
    Json(req): Json<DreamGenerateRequest>,
) -> Json<serde_json::Value> {
    let mut s = state.lock().unwrap();

    if s.goals.is_empty() {
        return Json(serde_json::json!({
            "dreams": [],
            "count": 0,
            "active": false,
            "reason": "No goals exist."
        }));
    }

    if !req.force && s.actor.is_working(&s.goals) {
        return Json(serde_json::json!({
            "dreams": [],
            "count": 0,
            "active": false,
            "reason": "PDCA cycle active — dreaming suppressed. Pass {'force': true} to override."
        }));
    }

    let goals = s.goals.clone();
    let dreams = s.dreams.generate_dreams(&goals, req.count);
    Json(serde_json::json!({
        "dreams": dreams.iter().map(|d| serde_json::json!({
            "id": d.id,
            "source": d.source,
            "embedding": d.embedding,
            "grade": d.grade,
        })).collect::<Vec<_>>(),
        "count": dreams.len(),
        "active": true,
    }))
}

async fn dream_evaluate_handler(
    State(state): State<SharedState>,
    Json(req): Json<DreamEvaluateRequest>,
) -> Json<serde_json::Value> {
    let mut s = state.lock().unwrap();
    let accepted = s.dreams.evaluate_dream(&req.id, req.grade);
    Json(serde_json::json!({
        "accepted": accepted,
        "dream_id": req.id,
        "grade": req.grade,
    }))
}

async fn coherence_snapshot_handler(
    State(state): State<SharedState>,
) -> Json<CoherenceSnapshot> {
    let s = state.lock().unwrap();
    Json(s.core.snapshot())
}

async fn coherence_register_handler(
    State(state): State<SharedState>,
    Json(req): Json<CoherenceRegisterRequest>,
) -> Json<CoherenceRegisterResponse> {
    let mut s = state.lock().unwrap();
    let embedding = s.embedder.embed(&req.input);
    let node_id = s.core.register_node_vec(embedding);
    Json(CoherenceRegisterResponse { node_id })
}

async fn context_filter_handler(
    State(state): State<SharedState>,
    Json(req): Json<ContextFilterRequest>,
) -> Json<FilterResponse> {
    let s = state.lock().unwrap();
    let result = s.core.filtra_contesto(&req.input, &s.embedder);
    Json(FilterResponse {
        embedding: result.embedding,
        valid: result.valid,
        token_estimate: result.token_estimate,
    })
}

async fn compress_logs_handler(
    State(state): State<SharedState>,
    Json(req): Json<CompressLogsRequest>,
) -> Json<CompressResponse> {
    let s = state.lock().unwrap();
    let compressed = s.core.compress_logs(&req.logs);
    Json(CompressResponse {
        input_count: req.logs.len(),
        output_chars: compressed.len(),
        compressed,
    })
}

async fn ingest_prompt_handler(
    State(state): State<SharedState>,
    Json(req): Json<serde_json::Value>,
) -> Json<IngestResponse> {
    let prompt = req.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let mut s = state.lock().unwrap();
    let embedding = s.embedder.embed(&prompt);
    let mut goal = Goal::new_vec(embedding);
    goal.progress = 0.0;
    s.goals.push(goal.clone());
    Json(IngestResponse {
        message: format!("Goal created from prompt ({} chars)", prompt.len()),
        goal,
    })
}

async fn pdca_plan_handler(
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let s = state.lock().unwrap();
    let planned = s.actor.plan(&s.goals);
    let goals: Vec<serde_json::Value> = planned.iter().map(|g| {
        serde_json::json!({
            "id": g.id,
            "embedding": g.embedding,
            "progress": g.progress,
        })
    }).collect();
    Json(serde_json::json!({ "planned": goals, "count": goals.len() }))
}

async fn pdca_act_handler(
    State(state): State<SharedState>,
    Json(req): Json<PDCARequest>,
) -> Json<serde_json::Value> {
    let (before, after) = {
        let s = state.lock().unwrap();
        let goal = s.goals.iter().find(|g| g.id == req.goal_id);
        match goal {
            Some(g) => (g.embedding.clone(), req.state_vector),
            None => return Json(serde_json::json!({"error": "goal not found"})),
        }
    };

    let exp_id = {
        let mut s = state.lock().unwrap();
        s.actor.do_action(&req.goal_id, before, after.clone()).id
    };

    let exps = {
        let s = state.lock().unwrap();
        s.actor.experiences.clone()
    };
    {
        let mut s = state.lock().unwrap();
        let mut goals = std::mem::take(&mut s.goals);
        s.actor.act(&exps, &mut goals);
        s.goals = goals;
    }

    Json(serde_json::json!({
        "experience_id": exp_id,
    }))
}

async fn pdca_stats_handler(
    State(state): State<SharedState>,
) -> Json<physis::actor::PDCAStats> {
    let s = state.lock().unwrap();
    Json(s.actor.stats(&s.goals))
}

async fn reconstruct_handler(
    State(state): State<SharedState>,
    Json(req): Json<ContextFilterRequest>,
) -> Json<physis::Reconstruction> {
    let s = state.lock().unwrap();
    let rec = physis::reconstruct(&req.input, &s.embedder, &s.core, 5);
    Json(rec)
}

// ── Main ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();

    let config = PhysisConfig::default();
    let ontology = OntologyLoader::load_all(&config);
    let mapper = OntologyMapper::new(ontology.clone());
    let actor = PDCActor::new(config.pdca_stagnant_threshold, config.pdca_stagnant_window);
    let dreams = DreamEngine::new();
    let core = PhysisCore::new();
    let embedder = RandomProjectionEmbedder::new(384);

    let state = Arc::new(Mutex::new(AppState {
        core,
        mapper,
        actor,
        dreams,
        ontology,
        goals: Vec::new(),
        embedder,
    }));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/stats", get(stats_handler))
        .route("/api/v1/query", get(query_handler))
        .route("/api/v1/scan", post(scan_handler))
        .route("/api/v1/dream/generate", post(dream_generate_handler))
        .route("/api/v1/dream/evaluate", post(dream_evaluate_handler))
        .route("/api/v1/coherence/snapshot", get(coherence_snapshot_handler))
        .route("/api/v1/coherence/register", post(coherence_register_handler))
        .route("/api/v1/context/filter", post(context_filter_handler))
        .route("/api/v1/compress/logs", post(compress_logs_handler))
        .route("/api/v1/goals", post(ingest_prompt_handler))
        .route("/api/v1/pdca/plan", get(pdca_plan_handler))
        .route("/api/v1/pdca/act", post(pdca_act_handler))
        .route("/api/v1/pdca/stats", get(pdca_stats_handler))
        .route("/api/v1/reconstruct", post(reconstruct_handler))
        .with_state(state);

    let port = std::env::var("PHYSIS_PORT").unwrap_or_else(|_| "19876".to_string());
    let addr = format!("127.0.0.1:{}", port);
    println!("Physis Web API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
