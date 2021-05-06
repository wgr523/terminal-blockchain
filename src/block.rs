use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::{Display, Formatter, Result};

#[derive(Serialize, Deserialize, Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct Block {
    pub miner: u8,
    pub number: u64,
    pub timestamp: u64,
    pub parent: Vec<u8>,
    // fake for now
    pub creator_signature: Vec<u8>,
    // fake for now
    pub verifier_signature: Option<Vec<u8>>,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let parent = hex::encode(&self.parent);
        if parent.len() >= 4 {
            write!(f, "miner: {}, parent: {}", self.miner, &parent[..4])
        } else {
            write!(f, "miner: {}, parent: null", self.miner)
        }
    }
}

impl Block {
    pub fn digest(&self) -> Vec<u8> {
        let mut no_verifier_signature = self.clone();
        no_verifier_signature.verifier_signature = None;
        let serialized = bincode::serialize(self).unwrap();
        let digest = ring::digest::digest(&ring::digest::SHA256, &serialized);
        digest.as_ref().to_vec()
    }

    pub fn new(miner: u8, parent: &Block) -> Self {
        let number = parent.number +1;
        Self {
            miner,
            number,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            parent: parent.digest(),
            creator_signature: vec![],
            verifier_signature: None,
        }
    }
    pub fn genesis() -> Self {
        Self {
            miner: 0,
            number: 0,
            timestamp: 10101,
            parent: vec![],
            creator_signature: vec![],
            verifier_signature: None,
        }
    }
}