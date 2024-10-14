use blake2::{Blake2s256, Digest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MerkleTree {
    root: Option<Vec<u8>>,
    nodes: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Order {
    Before,
    After,
}

impl MerkleTree {
    pub fn new(data: &[Vec<u8>]) -> MerkleTree {
        let mut nodes = Vec::new();
        let mut hasher = Blake2s256::new();
        let mut leaves: Vec<Vec<u8>> = data
            .iter()
            .map(|d| {
                hasher.update(d);
                hasher.finalize_reset().to_vec()
            })
            .collect();
        let ceil = leaves.len().next_power_of_two();

        while leaves.len() < ceil {
            leaves.push(vec![]);
        }
        nodes.extend(leaves.clone());

        while leaves.len() > 1 {
            leaves = leaves
                .chunks(2)
                .map(|chunk| {
                    hasher.update(&chunk[0]);
                    hasher.update(&chunk[1]);
                    hasher.finalize_reset().to_vec()
                })
                .collect();
            nodes.extend(leaves.clone());
        }

        MerkleTree {
            root: leaves.first().cloned(),
            nodes,
        }
    }

    pub fn merkle_proof(&self, mut i: usize) -> Vec<(Vec<u8>, Order)> {
        let mut proof = Vec::new();
        let mut level_size = 0;
        let mut level_nodes = (self.nodes.len() + 1) / 2;

        loop {
            let (order, sibling) = if i % 2 == 0 {
                (Order::After, i + 1)
            } else {
                (Order::Before, i - 1)
            };

            if let Some(node) = self.nodes.get(level_size + sibling) {
                proof.push((node.clone(), order));
            } else {
                break;
            }

            level_size += level_nodes;
            level_nodes = (level_nodes + 1) / 2;
            i /= 2;
        }
        proof
    }

    pub fn root_from_proof(file: &[u8], proof: Vec<(Vec<u8>, Order)>) -> Vec<u8> {
        let mut hasher = Blake2s256::new();
        hasher.update(file);
        let file_node = hasher.finalize_reset();

        proof
            .iter()
            .fold(file_node, |acc, (sibling, order)| {
                match order {
                    Order::Before => {
                        hasher.update(sibling);
                        hasher.update(acc);
                    }
                    Order::After => {
                        hasher.update(acc);
                        hasher.update(sibling);
                    }
                }
                hasher.finalize_reset()
            })
            .to_vec()
    }

    pub fn root(&self) -> Option<&Vec<u8>> {
        self.root.as_ref()
    }
}

#[test]
fn basics_randomized() {
    use rand::{Rng, RngCore};

    let upper = rand::thread_rng().gen_range(1..512);
    let i = rand::thread_rng().gen_range(0..upper);
    let data: Vec<Vec<u8>> = (0..upper)
        .map(|_| {
            let mut data = [0u8; 8];
            rand::thread_rng().fill_bytes(&mut data);
            data.into()
        })
        .collect();

    let mtree = MerkleTree::new(&data);
    let root = mtree.root().unwrap();
    let proof_i = mtree.merkle_proof(i);

    assert_eq!(*root, MerkleTree::root_from_proof(&data[i], proof_i));
}
