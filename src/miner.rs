use crossterm::{cursor, Result};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType};
use crossterm::style::{Color, SetForegroundColor, SetBackgroundColor, Print};

use std::sync::{Arc, RwLock};
use crate::block_tree::BlockTree;
use std::sync::mpsc::{Sender, Receiver, channel};
use crate::block::Block;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::io::stdout;

pub struct Miner {
    id: u8,
    n: u8,
    sleep_ms: u64,
    block_int_ms: u64,
    my_turn_wait_ms: u64,
    block_tree: Arc<RwLock<BlockTree>>,
    to_network: Sender<Block>,
    from_network: Receiver<Block>,
}

impl Miner {
    pub fn new(id: u8, n: u8, to_network: Sender<Block>, from_network: Receiver<Block>) -> (Miner, Arc<RwLock<BlockTree>>) {
        let block_tree = BlockTree::default();
        let block_tree = Arc::new(RwLock::new(block_tree));
        let bt_clone = block_tree.clone();
        let miner = Miner {
            id,
            n,
            sleep_ms: 100,
            block_int_ms: 2000,
            my_turn_wait_ms: 10000,
            block_tree,
            to_network,
            from_network
        };
        (miner, bt_clone)
    }

    pub fn start(mut self) {
        std::thread::Builder::new().name(format!("Miner {}", self.id)).spawn(move || self.miner_loop()).unwrap();
    }

    fn miner_loop(&mut self) {
        loop {
            if let Ok(block) = self.from_network.try_recv() {
                let mut store = self.block_tree.write().unwrap();
                store.insert(block);
            } else {
                std::thread::sleep(Duration::from_millis(self.sleep_ms));
            }
            let parent = {
                let store = self.block_tree.read().unwrap();
                // if no genesis, should wait until genesis comes
                if store.number_block.is_empty() {
                    continue;
                }
                store.tip.clone()
            };
            if self.id == parent.miner {
                continue;
            }
            // my turn to mine
            if self.id == (parent.miner + 1) % self.n {
                let expect_timestamp = UNIX_EPOCH + Duration::from_secs(parent.timestamp) + Duration::from_millis(self.block_int_ms);
                if let Ok(duration) = expect_timestamp.duration_since(SystemTime::now()) {
                    std::thread::sleep(duration);
                }
                let block = Block::new(self.id, &parent);
                {
                    let mut store = self.block_tree.write().unwrap();
                    store.insert(block.clone());
                }
                self.to_network.send(block).unwrap();
            } else {
                // should skip the one just next to genesis
                if parent.timestamp == 10101 {
                    continue;
                }
                // not my turn, wait
                let h = hop(self.n, parent.miner, self.id);
                let gap = h as u64 * self.my_turn_wait_ms;
                let expect_timestamp = UNIX_EPOCH + Duration::from_secs(parent.timestamp) + Duration::from_millis(gap);
                if expect_timestamp > SystemTime::now() {
                    continue;
                }
                let block = Block::new(self.id, &parent);
                {
                    let mut store = self.block_tree.write().unwrap();
                    store.insert(block.clone());
                }
                self.to_network.send(block).unwrap();
            }
        }
    }
}

fn hop(n: u8, pre: u8, cur: u8) -> u8 {
    if pre < cur {
        cur - (pre + 1)
    } else if pre > cur {
        n + cur - (pre + 1)
    } else {
        n - 1
    }
}
