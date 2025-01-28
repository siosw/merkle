// array of values
// root() -> calculate merke root
// proof(index) -> return proof for index

use std::{
    collections::VecDeque,
    hash::{DefaultHasher, Hash, Hasher},
};

use eyre::OptionExt;

#[derive(Debug)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
struct Step {
    direction: Direction,
    value: u64,
}

/// proof that leaf is included in a tree with the given root
#[derive(Debug)]
struct MerkleProof {
    path: Vec<Step>,
    root: u64,
    leaf: u64,
}

struct MerkleTree<T> {
    values: Vec<T>,
}

impl<T> From<Vec<T>> for MerkleTree<T>
where
    T: Hash + Clone,
{
    fn from(values: Vec<T>) -> Self {
        Self { values }
    }
}

impl<T> MerkleTree<T>
where
    T: Hash + Default + Clone + Copy,
{
    pub fn add(&mut self, value: T) {
        self.values.push(value);
    }

    /// return the leafs of the merkle tree
    /// the leafs are hashed values or hashed default values
    /// the number of leafs always equals the smallest power of two that is greater
    /// than the number of values stored in the tree
    pub fn leafs(&self) -> VecDeque<u64> {
        let size = self.values.len().next_power_of_two();
        let empty = vec![T::default(); size - self.values.len()];
        let mut leafs = self.values.clone();
        leafs.extend(empty);

        leafs.iter().map(Self::hash_leaf).collect()
    }

    pub fn root(&self) -> u64 {
        let mut hashes = self.leafs();
        while hashes.len() > 1 {
            Self::parents(&mut hashes);
        }

        hashes.pop_front().unwrap()
    }

    pub fn get_proof(&self, index: usize) -> eyre::Result<MerkleProof> {
        let mut index = index;
        let mut hashes = self.leafs();
        let leaf = *hashes.get(index).ok_or_eyre("index out of bounds")?;

        let mut proof = MerkleProof {
            leaf,
            root: self.root(),
            path: Vec::new(),
        };

        while hashes.len() > 1 {
            let sibling = if index % 2 == 0 { index + 1 } else { index - 1 };
            let direction = if index > sibling {
                Direction::Left
            } else {
                Direction::Right
            };
            let value = hashes.get(sibling).unwrap().clone();
            proof.path.push(Step { direction, value });

            Self::parents(&mut hashes);
            index = index / 2;
        }

        Ok(proof)
    }

    pub fn verify_proof(proof: &MerkleProof) -> bool {
        let mut acc = proof.leaf;

        for step in &proof.path {
            acc = match step.direction {
                Direction::Left => Self::hash_siblings(&step.value, &acc),
                Direction::Right => Self::hash_siblings(&acc, &step.value),
            }
        }

        proof.root == acc
    }

    fn parents(hashes: &mut VecDeque<u64>) {
        let len = hashes.len();

        assert_eq!(len % 2, 0);
        assert!(len > 0);

        for _ in 0..(len / 2) {
            let left = hashes.pop_front().unwrap();
            let right = hashes.pop_front().unwrap();
            let parent = Self::hash_siblings(&left, &right);
            hashes.push_back(parent);
        }
    }

    fn hash_leaf(leaf: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        leaf.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_siblings(left: &u64, right: &u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        left.hash(&mut hasher);
        right.hash(&mut hasher);
        hasher.finish()
    }
}

fn main() {
    let values: Vec<u32> = (0..4).collect();
    let mut tree = MerkleTree::from(values);
    tree.add(4);

    println!("leafs: {:?}", tree.leafs());

    let root = tree.root();
    println!("root {}", root);

    let p = tree.get_proof(2).unwrap();
    println!("proof: {:?}", p);

    println!("valid: {:?}", MerkleTree::<u32>::verify_proof(&p));
}

#[test]
fn basic_proof() -> eyre::Result<()> {
    let values: Vec<u32> = (0..100_000).collect();
    let mut tree = MerkleTree::from(values);
    let proof = tree.get_proof(500)?;

    assert!(MerkleTree::<u32>::verify_proof(&proof));
    assert_eq!(&proof.root, &tree.root());

    tree.add(42);
    assert_ne!(&proof.root, &tree.root());

    Ok(())
}

#[test]
fn empty_tree() -> eyre::Result<()> {
    let tree: MerkleTree<u32> = MerkleTree::from(vec![]);

    // tree initiated with an empty list should have 1 leaf which is also the root
    assert_eq!(tree.leafs().len(), 1);

    let proof = tree.get_proof(0)?;
    assert!(MerkleTree::<u32>::verify_proof(&proof));
    assert_eq!(&proof.root, &tree.root());

    Ok(())
}

#[test]
fn out_of_bounds() {
    let tree: MerkleTree<u32> = MerkleTree::from(vec![1, 2]);
    assert_eq!(tree.leafs().len(), 2);

    let proof = tree.get_proof(2);
    assert!(proof.is_err());
}
