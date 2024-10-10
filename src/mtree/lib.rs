extern crate blake2;

use blake2::{Blake2s256, Digest};

#[derive(Debug, Clone)]
struct MerkleTree {
    root: Option<String>,
    nodes: Vec<String>,
}

impl MerkleTree {
    pub fn new(data: Vec<String>) -> MerkleTree {
        let mut nodes = Vec::new();
        let mut leaves: Vec<String> = data
            .into_iter()
            .map(|d| {
                let mut hasher = Blake2s256::new();
                hasher.update(d.as_bytes());
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
            root: leaves.get(0).cloned(),
            nodes,
        }
    }

    pub fn root(&self) -> Option<&String> {
        self.root.as_ref()
    }
}

fn main() {
    let data = vec![
        "data1".to_string(),
        "data2".to_string(),
        "data3".to_string(),
        "data4".to_string(),
        "data5".to_string(),
        "data6".to_string(),
        "data7".to_string(),
        "data8".to_string(),
        "data9".to_string(),
    ];

    let tree = MerkleTree::new(data);

    println!("Merkle Tree Root: {:?}", tree.root());
    println!("Merkle Tree Nodes: {:?}", tree.nodes);
    println!("Merkle Tree Nodes Len: {:?}", tree.nodes.len());
}
