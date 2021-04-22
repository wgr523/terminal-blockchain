use crossterm::{cursor, Result};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType};
use crossterm::style::{Color, SetForegroundColor, SetBackgroundColor, Print};

use std::sync::mpsc::{Receiver, channel, Sender};
use crate::block::Block;
use crate::block_tree::BlockTree;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::io::stdout;
use crate::draw_block_hash;
use std::time::Duration;

pub struct Network {
    pub n: u8,
    pub from_miners: Receiver<Block>,
    pub senders: HashMap<u8, Sender<Block>>,
    pub artificial_delay: HashMap<(u8,u8), u64>,
}


impl Network {

    pub fn set_delay(&mut self, from: u8, to: u8, delay: u64) {
        let d = self.artificial_delay.entry((from, to)).or_default();
        *d = delay;
    }

    pub fn genesis(&self)  -> Result<()> {
        let block = Block::genesis();
        for id in 0..self.n {
            if let Some(sender) = self.senders.get(&id) {
                sender.send(block.clone()).unwrap();
            }
        }
        Ok(())
    }

    fn main_loop(&self)  -> Result<()> {
        self.genesis()?;
        loop {
            let block = self.from_miners.recv().unwrap();
            for id in 0..self.n {
                if id == block.miner {
                    continue;
                }
                if let Some(sender) = self.senders.get(&id) {
                    if let Some(d) = self.artificial_delay.get(&(block.miner, id)) {
                        let d = *d;
                        let sender = sender.clone();
                        let block = block.clone();
                        std::thread::Builder::new().name(format!("network artificial delay")).spawn(move || {
                            std::thread::sleep(Duration::from_millis(d));
                            sender.send(block).unwrap();
                        }).unwrap();
                    } else {
                        sender.send(block.clone()).unwrap();
                    };
                }
            }
        }
    }

    pub fn start(self) {
        std::thread::Builder::new().name(format!("network")).spawn(move || self.main_loop()).unwrap();
    }
}