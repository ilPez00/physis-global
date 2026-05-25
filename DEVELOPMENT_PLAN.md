# Physis Engine — Development Plan

## Overview

Physis is moving from a single-threaded ontological mapper into a **holarchic runtime**
with real-time graph processing, semantic classification, temporal scheduling, and
distributed dream synchronization.

## Phase 1: Holarchy & Real-time Graph (Current — In Progress)

### Graph Engine (`graph.rs`)
- `DenseSlotMap<NodeKey, NodePayload>` with `Pod`-compatible zero-copy types
- `Edge` with semantic type and weight — `MmappedStorage` for persist
- ONNX worker thread processes graph nodes asynchronously via `crossbeam_channel`
- `RawNodeKey` bridges graph internals with serialization boundaries

**Status**: DenseSlotMap, Pod types, Edge, NodePayload, MmappedStorage, ONNX worker channel — done.
**Next**: Graph traversal (BFS/DFS), subgraph extraction, causal path search.

### Rachmaninov Holon (`rachmaninov.rs`)
- PDCA-driven directive engine: `Focus`, `Expand`, `Prune`, `Synthesize` directives
- `tick(state_vector)` compares state vs goal vector, emits directives on drift
- Designed as a control holon that reads graph state and writes directives back

**Status**: Core tick cycle, directive types, similarity threshold gating — done.
**Next**: Wire tick output into graph mutation pipeline, add directive priority sorting.

### Sensory Pipeline (`sensory.rs`)
- Raw sensor payloads as `Pod` structs — `SensorPayload` with timestamp + f32x4 vector
- Bridge between real-world input (filesystem, network, audio) and graph node space
- Packed binary format for zero-copy mmap storage

**Status**: Data types, serialization — done.
**Next**: Ingestion from audio/network sources, real-time node creation on sensor fire.

### ONNX Worker (`ai/onnx_worker.rs`)
- Background thread pool processing embedding requests
- Request/response channel pattern — non-blocking for the web server
- Pool of active `NdTensor` views for batch inference

**Status**: Thread pool, channel plumbing, ndarray shape fix (3D → flat index) — done.
**Next**: Batch queue with priority, streaming inference, GPU fallback detection.

## Phase 2: Semantic Embedding & Classification

### ONNX MiniLM (done)
- Production-grade 384-d embeddings via `ort` runtime + `tokenizers` crate
- Falls back to deterministic `RandomProjectionEmbedder` when model absent
- Enabled with `--features embed-onnx`

### Query Classification (done)
- `POST /api/v1/classify` — embed query text → cosine sim against per-cell centroids
- Centroids pre-computed at startup from all 511 ontology entries (name + hints averaged per DOMAIN×MODE cell)
- Returns sorted `ClassifyResult[]` with domain, mode, score, matched ontology entries

**Next**: Image classification (extend embedder for multi-modal), process/event classification (task graph → embedding), class weight tuning from feedback.

## Phase 3: Gantt & Temporal Scheduling

### Gantt Engine (`gantt.rs`)
- `GanttTask` with start/end, dependencies, resource allocation
- `RawNodeKey` integration — tasks reference graph nodes
- Critical path analysis and resource leveling

**Status**: Data structures — done (stub).
**Next**: Causal-aware scheduling (failures push dates, successes pull them in), temporal coherence scoring, Gantt → Mermaid renderer.

## Phase 4: Distributed Physis

### MCP Mesh
- Global UberWiki sync between instances via MCP
- `physis_ingest_prompt` → local goal → remote broadcast
- Dream pattern sharing across instances

**Status**: MCP server/client stubs — done.
**Next**: Conflict resolution for concurrent writes, partial sync (subscribe to branches), latency-tolerant dream merging.

## Phase 5: Advanced Dreaming

### Graph-based Dreams
- Dream generation traverses graph edges instead of linear vector interpolation
- ONNXWorker-assisted dream evaluation (semantic plausibility scoring)
- Dream → subgraph extraction → directive queue

**Status**: Linear stochastic dreams (mutation/graft/prune) — done.
**Next**: Graph-aware mutation (prune failed subgraphs, expand coherent ones), predictive coherence simulation, cross-instance dream comparison.

## Milestone Dependencies

```
Phase 1 ──┬──> Phase 2 ──> Phase 3
           │
           └──> Phase 4 ──> Phase 5
```

Phase 2 can proceed in parallel with Phase 1 (mostly disjoint modules).
Phase 3 depends on Phase 1 graph traversal.
Phase 4 depends on Phase 1 MCP stability + Phase 2 classification quality.
Phase 5 depends on Phase 1 graph engine + Phase 4 distributed state.

## Key Metrics

| Area | Metric | Target |
|------|--------|--------|
| Classification | Top-1 accuracy on curated test set | >0.85 |
| Graph throughput | Node mutations/sec | >10_000 |
| Dream quality | Human eval acceptance rate | >0.6 |
| Sync latency | P95 MCP broadcast to N instances | <500ms |
| Embedding latency | P99 ONNX embed (text) | <50ms |
