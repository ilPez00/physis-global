use std::collections::HashMap;
use std::sync::Mutex;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use sha2::{Sha256, Digest};

pub trait VectorEmbed: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
    fn embed_batch(&self, texts: &[&str]) -> Vec<Vec<f32>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    fn dimension(&self) -> usize;
}

fn hash_ngrams(text: &str, n: usize) -> Vec<u64> {
    let padded = format!(" {text} ");
    let chars: Vec<char> = padded.chars().collect();
    let mut hashes = Vec::new();
    for i in 0..chars.len().saturating_sub(n - 1) {
        let gram: String = chars[i..i + n].iter().collect();
        let mut h = Sha256::new();
        h.update(gram.as_bytes());
        let result = h.finalize();
        hashes.push(u64::from_le_bytes(result[..8].try_into().unwrap()));
    }
    hashes
}

fn random_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut v = Vec::with_capacity(dim);
    for _ in 0..dim {
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        v.push(z as f32);
    }
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
    v.iter_mut().for_each(|x| *x /= norm);
    v
}

/// Deterministic random-projection embedder using feature hashing.
/// Maps any text to a fixed-dimension vector (384 by default).
/// Same input always produces the same vector.
pub struct RandomProjectionEmbedder {
    dim: usize,
    seed: u64,
    basis: Mutex<HashMap<u64, Vec<f32>>>,
}

impl RandomProjectionEmbedder {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            seed: 42,
            basis: Mutex::new(HashMap::new()),
        }
    }

    fn get_basis(&self, key: u64) -> Vec<f32> {
        let mut cache = self.basis.lock().unwrap();
        cache.entry(key).or_insert_with(|| random_vector(self.dim, key ^ self.seed)).clone()
    }
}

impl VectorEmbed for RandomProjectionEmbedder {
    fn embed(&self, text: &str) -> Vec<f32> {
        let lower = text.to_lowercase();
        let mut vec = vec![0.0f32; self.dim];

        for h in hash_ngrams(&lower, 1) {
            let basis = self.get_basis(h);
            for (i, val) in basis.iter().enumerate() {
                vec[i] += val;
            }
        }
        for h in hash_ngrams(&lower, 2) {
            let basis = self.get_basis(h ^ 0xFFFF);
            for (i, val) in basis.iter().enumerate() {
                vec[i] += val * 0.5;
            }
        }

        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        vec.iter_mut().for_each(|x| *x /= norm);
        vec
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

#[cfg(feature = "embed-onnx")]
pub mod onnx;
