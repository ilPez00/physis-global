use slotmap::{DenseSlotMap, new_key_type, Key, KeyData};
use serde::{Serialize, Deserialize};
use bytemuck::{Pod, Zeroable};
use crossbeam_channel::{Sender, unbounded};
use crate::ai::onnx_worker::{OnnxRequest, OnnxResponse};
use crate::storage::MmappedStorage;

new_key_type! {
    pub struct NodeKey;
}

/// Pod-compatible key representation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Serialize, Deserialize)]
pub struct RawNodeKey {
    pub data: u64,
}

impl From<NodeKey> for RawNodeKey {
    fn from(key: NodeKey) -> Self {
        Self { data: key.data().as_ffi() }
    }
}

impl From<RawNodeKey> for NodeKey {
    fn from(raw: RawNodeKey) -> Self {
        KeyData::from_ffi(raw.data).into()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Serialize, Deserialize)]
pub struct NodePayload {
    pub activation_energy: f32,
    pub decay_metrics: f32,
    pub prune_flag: u8,
    pub compress_flag: u8,
    pub _padding: [u8; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Serialize, Deserialize)]
pub struct Edge {
    pub source: RawNodeKey,
    pub target: RawNodeKey,
    pub weight: f32,
    pub semantic_type: u32,
}

pub struct HolonicGraph {
    pub nodes: DenseSlotMap<NodeKey, NodePayload>,
    pub edges: Vec<Edge>,
    pub embeddings: DenseSlotMap<NodeKey, Vec<f32>>,
}

impl HolonicGraph {
    pub fn new() -> Self {
        Self {
            nodes: DenseSlotMap::with_key(),
            edges: Vec::new(),
            embeddings: DenseSlotMap::with_key(),
        }
    }

    pub fn add_node(&mut self, payload: NodePayload, embedding: Vec<f32>) -> NodeKey {
        let key = self.nodes.insert(payload);
        self.embeddings.insert_with_key(|_| embedding);
        key
    }

    pub fn save_to_mmap(&self, storage: &mut MmappedStorage) -> anyhow::Result<()> {
        let nodes_vec: Vec<NodePayload> = self.nodes.values().cloned().collect();
        let nodes_bytes = bytemuck::cast_slice(&nodes_vec);
        if nodes_bytes.len() <= storage.mmap.len() {
            storage.mmap[0..nodes_bytes.len()].copy_from_slice(nodes_bytes);
        }
        Ok(())
    }
}

/// Zero-copy accessor for the memory-mapped graph file.
pub struct MmappedGraph<'a> {
    pub storage: &'a mut MmappedStorage,
}

impl<'a> MmappedGraph<'a> {
    pub fn new(storage: &'a mut MmappedStorage) -> Self {
        Self { storage }
    }

    pub fn nodes_mut(&mut self) -> &mut [NodePayload] {
        let header = *self.storage.header_mut();
        let offset = std::mem::size_of::<crate::storage::GraphHeader>();
        self.storage.as_slice_mut(offset, header.node_count as usize)
    }

    pub fn edges_mut(&mut self) -> &mut [Edge] {
        let header = *self.storage.header_mut();
        let node_offset = std::mem::size_of::<crate::storage::GraphHeader>();
        let edge_offset = node_offset + (header.node_count as usize * std::mem::size_of::<NodePayload>());
        self.storage.as_slice_mut(edge_offset, header.edge_count as usize)
    }
}

/// The Ingest Holon entry point.
pub struct IngestRing {
    tx: Sender<Vec<u8>>,
}

impl IngestRing {
    pub fn spawn(onnx_tx: Sender<OnnxRequest>, graph_tx: Sender<(NodePayload, Vec<f32>)>) -> Self {
        let (tx, rx) = unbounded::<Vec<u8>>();
        
        std::thread::spawn(move || {
            let (reply_tx, reply_rx) = unbounded::<OnnxResponse>();
            while let Ok(buffer) = rx.recv() {
                let _ = onnx_tx.send(OnnxRequest::IngestFilter {
                    raw_buffer: buffer,
                    reply_to: reply_tx.clone(),
                });
                
                if let Ok(OnnxResponse::IngestResult { cleaned_text: _, payload }) = reply_rx.recv() {
                    let dummy_embedding = vec![0.0; 32];
                    let _ = graph_tx.send((payload, dummy_embedding));
                }
            }
        });
        
        Self { tx }
    }

    pub fn push(&self, data: Vec<u8>) {
        let _ = self.tx.send(data);
    }
}
