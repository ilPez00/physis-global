# Physis — AI Engine

## `/mcp` — Toggle Physis MCP
Toggles the Physis engine (Rust HTTP API) on/off.

```
/mcp             — toggle (start if stopped, stop if running)
/mcp on          — start the Physis engine on :19876
/mcp off         — stop the Physis engine
/mcp status      — check if Physis is running
```

The script: `physis/bin/physis-toggle`

When running, `physis_*` MCP tools are available. When stopped, tool calls will fail gracefully.

## Paradigm
Physis operates in **pure vector space**. All data structures are vectors:
- Goals: `{id, embedding: Vec<f32>, progress: f32}` — no name, no domain, no labels
- Coherence nodes: `{id, embedding: Vec<f32>, coherence_score: f32}` — no rating, no axis, no domain
- Experiences: `{id, goal_id, before: Vec<f32>, after: Vec<f32>, grade: f32}` — no action, no rationale
- Dreams: `{id, source: Vec<f32>, embedding: Vec<f32>, grade: Option<f32>}` — no type, no description, no variation

Text is embedded server-side using RandomProjectionEmbedder (384-d, deterministic). Semantic quality can be upgraded via the `embed-onnx` feature (ONNX MiniLM).

## Prompt Ingest
When the user gives you a task, request, or instruction, call `physis_ingest_prompt` with the prompt text to register it as a vector Goal in the Physis engine. This enables PDCA tracking, dreaming, and coherence evaluation.

## MCP Tools
- `physis_ingest_prompt(prompt)` — register a user prompt as a vector Goal
- `physis_health()` — check if the engine is running
- `physis_stats()` — get trie/coherence/domain statistics (vector band counts, no qualia)
- `physis_query(query, max_results)` — search the UberWiki trie
- `physis_scan(dir)` — index a directory into the trie as vector Goals
- `physis_dream_generate(count, force)` — generate vector-space dreams (interpolate/extrapolate/mutate)
- `physis_dream_evaluate(dream_id, grade)` — accept/reject a dream
- `physis_coherence_snapshot()` — get geometric coherence snapshot (bands: high/mid/low)
- `physis_coherence_register(input)` — register a coherence node (embedding server-side)
- `physis_context_filter(input)` — filter text through geometric context filter
- `physis_compress_logs(logs)` — compress logs into causal rules
- `physis_pdca_plan()` — get the PDCA plan (goals sorted by progress)
- `physis_pdca_act(goal_id, state_vector)` — execute a PDCA action (state transition vector)
- `physis_pdca_stats()` — get PDCA statistics (actions, progress, stagnant count)
- `physis_reconstruct(input)` — find nearest neighbor vectors for a query
