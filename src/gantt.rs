use serde::{Serialize, Deserialize};
use crate::graph::RawNodeKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct GanttTask {
    pub id: String,
    pub node_id: RawNodeKey,
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub progress: f32,
    pub dependencies: Vec<RawNodeKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trajectory {
    pub tasks: Vec<GanttTask>,
    pub coherence_index: f32,
}

pub struct GanttHolon;

impl GanttHolon {
    pub fn compute_trajectory(graph: &crate::graph::HolonicGraph) -> Trajectory {
        let mut tasks = Vec::new();
        
        // Map graph nodes to tasks based on activation energy and edges
        for (key, payload) in graph.nodes.iter() {
            let raw_key = RawNodeKey::from(key);
            
            // Dummy temporal mapping: logic based on graph structure
            tasks.push(GanttTask {
                id: format!("task_{}", raw_key.data),
                node_id: raw_key,
                name: format!("Holon {}", raw_key.data),
                start_time: 0, 
                end_time: 100,
                progress: payload.activation_energy.clamp(0.0, 1.0),
                dependencies: graph.edges.iter()
                    .filter(|e| e.target.data == raw_key.data)
                    .map(|e| e.source)
                    .collect(),
            });
        }

        Trajectory {
            tasks,
            coherence_index: 1.0, // Placeholder
        }
    }
}
