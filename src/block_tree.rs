use std::collections::{HashMap, HashSet};
use super::block::Block;


#[derive(Default)]
pub struct BlockTree {
    pub number_block: HashMap<u64, HashSet<Block>>,
    pub tip: Block,
}

impl BlockTree {
    pub fn insert(&mut self, block: Block) {
        match self.number_block.get_mut(&block.number) {
            Some(v) => {
                v.insert(block);
            }
            None => {
                let number = block.number;
                self.tip = block.clone();
                let mut v = HashSet::new();
                v.insert(block);
                self.number_block.insert(number, v);
            }
        };
    }
}