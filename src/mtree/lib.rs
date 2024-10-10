extern crate blake2;

use blake2::{Blake2s256, Digest};

#[derive(Debug, Clone)]
pub struct MerkleTree {
    root: Option<String>,
    _nodes: Vec<String>,
}

impl MerkleTree {
    pub fn new(data: &[Vec<u8>]) -> MerkleTree {
        let mut nodes = Vec::new();
        let mut leaves: Vec<String> = data
            .iter()
            .map(|d| {
                let mut hasher = Blake2s256::new();
                hasher.update(d);
                let result = hasher.finalize_reset();
                format!("{:x}", result)
            })
            .collect();

        if leaves.len() % 2 != 0 {
            leaves.push(leaves.last().unwrap().clone());
        }
        nodes.extend(leaves.clone());

        while leaves.len() > 1 {
            leaves = leaves
                .chunks(2)
                .map(|chunk| {
                    let mut hasher = Blake2s256::new();
                    hasher.update(chunk[0].as_bytes());
                    hasher.update(chunk[1].as_bytes());
                    let result = hasher.finalize_reset();
                    format!("{:x}", result)
                })
                .collect();

            if leaves.len() % 2 != 0 && leaves.len() > 1 {
                leaves.push(leaves.last().unwrap().clone());
            }
            nodes.extend(leaves.clone());
        }

        MerkleTree {
            root: leaves.first().cloned(),
            _nodes: nodes,
        }
    }

    pub fn root(&self) -> Option<&String> {
        self.root.as_ref()
    }
}
