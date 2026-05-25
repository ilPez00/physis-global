use crossbeam_channel::unbounded;
use physis::ai::onnx_worker::OnnxHolon;
use physis::graph::{HolonicGraph, IngestRing, NodePayload, RawNodeKey};
use physis::rachmaninov::RachmaninovHolon;

#[test]
fn test_holonic_graph_add_node() {
    let mut graph = HolonicGraph::new();
    let key = graph.add_node(
        NodePayload {
            activation_energy: 1.0,
            decay_metrics: 0.0,
            prune_flag: 0,
            compress_flag: 0,
            _padding: [0; 2],
        },
        vec![0.5; 32],
    );
    assert!(graph.nodes.contains_key(key));
    assert!(graph.embeddings.contains_key(key));
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_rayonix_holon_tick() {
    let mut rach = RachmaninovHolon::new();
    rach.tick(&[0.1, 0.2, 0.3]);
    // First tick initializes current_goal_vector, no directives
    assert!(rach.active_directives.is_empty());
    
    // Second tick with drift generates directive
    rach.tick(&[0.9, 0.8, 0.7]);
    assert!(!rach.active_directives.is_empty());
}

#[test]
fn test_ingest_ring_spawn_and_push() {
    let holon = OnnxHolon::spawn();
    let onnx_tx = holon.get_tx();
    let (graph_tx, graph_rx) = unbounded();
    let ring = IngestRing::spawn(onnx_tx, graph_tx);
    ring.push(3, b"hello world".to_vec());
    let received = graph_rx.recv_timeout(std::time::Duration::from_secs(5));
    assert!(received.is_ok(), "IngestRing should forward processed data to graph");
    if let Ok((payload, embedding)) = received {
        assert!((payload.activation_energy - 2.0).abs() < 1e-6); // 3 -> 2.0
        assert_eq!(embedding.len(), 32);
    }
}

#[test]
fn test_onnx_holon_spawn_and_ingest() {
    let holon = OnnxHolon::spawn();
    let (reply_tx, reply_rx) = unbounded();
    holon.send(physis::ai::onnx_worker::OnnxRequest::IngestFilter {
        signal_type: 0,
        raw_buffer: b"test data".to_vec(),
        reply_to: reply_tx,
    });
    let response = reply_rx.recv_timeout(std::time::Duration::from_secs(5));
    assert!(response.is_ok());
    if let Ok(physis::ai::onnx_worker::OnnxResponse::IngestResult { payload, .. }) = response {
        assert!((payload.activation_energy - 1.2).abs() < 1e-6); // 0 -> 1.2
    }
}

#[test]
fn test_raw_node_key_roundtrip() {
    use physis::graph::NodeKey;
    let mut graph = HolonicGraph::new();
    let key = graph.add_node(
        NodePayload {
            activation_energy: 0.5,
            decay_metrics: 0.1,
            prune_flag: 0,
            compress_flag: 0,
            _padding: [0; 2],
        },
        vec![0.0; 32],
    );
    let raw: RawNodeKey = key.into();
    let restored: NodeKey = raw.into();
    assert!(graph.nodes.contains_key(restored));
    assert_eq!(graph.nodes[restored].activation_energy, 0.5);
}
