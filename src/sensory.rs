use serde::{Serialize, Deserialize};
use bytemuck::{Pod, Zeroable};
use std::time::SystemTime;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SignalType {
    AudioPCM = 0,
    Acceleration = 1,
    Gyroscope = 2,
    ThoughtCapture = 3,
    VisionFeature = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct AuraFrameHeader {
    pub magic: [u8; 4], // 'AURA'
    pub _padding1: [u8; 4], // Align timestamp to 8
    pub timestamp: u64,
    pub signal_type: u8,
    pub _padding2: [u8; 3], // Align payload_len to 4
    pub payload_len: u32,
}

pub struct AuraFrame {
    pub header: AuraFrameHeader,
    pub payload: Vec<u8>,
}

impl AuraFrame {
    pub fn new(signal_type: SignalType, payload: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        Self {
            header: AuraFrameHeader {
                magic: *b"AURA",
                _padding1: [0; 4],
                timestamp,
                signal_type: signal_type as u8,
                _padding2: [0; 3],
                payload_len: payload.len() as u32,
            },
            payload,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = bytemuck::bytes_of(&self.header).to_vec();
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

pub mod listener {
    use tokio::net::UnixListener;
    use tokio::io::AsyncReadExt;
    use crate::graph::IngestRing;
    use std::sync::Arc;

    pub async fn start_sensory_server(path: &str, ingest: Arc<IngestRing>) -> anyhow::Result<()> {
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;
        
        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    let ingest_clone = ingest.clone();
                    tokio::spawn(async move {
                        let mut header_buf = [0u8; std::mem::size_of::<super::AuraFrameHeader>()];
                        while socket.read_exact(&mut header_buf).await.is_ok() {
                            let header: super::AuraFrameHeader = *bytemuck::from_bytes(&header_buf);
                            let mut payload = vec![0u8; header.payload_len as usize];
                            if socket.read_exact(&mut payload).await.is_ok() {
                                // Push raw payload to Physis Ingest Ring with signal type
                                ingest_clone.push(header.signal_type, payload);
                            }
                        }
                    });
                }
                Err(e) => log::error!("Sensory socket accept error: {}", e),
            }
        }
    }
}
