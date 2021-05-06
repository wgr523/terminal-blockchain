use crossterm::{cursor, Result};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType};
use crossterm::style::{Color, SetForegroundColor, SetBackgroundColor, Print};

use std::sync::mpsc::{Receiver, channel, Sender};
use crate::block::Block;
use crate::block_tree::BlockTree;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::io::stdout;
use std::time::Duration;
use std::hash::Hash;

pub struct Network {
    pub n: u8,
    pub from_miners: Receiver<Block>,
    pub senders: HashMap<u8, Sender<Block>>,
    pub artificial_delay: HashMap<(u8,u8), u64>,
    pub log: Arc<RwLock<String>>,
}


impl Network {

    pub fn set_single_delay(&mut self, from: u8, to: u8, delay: u64) {
        let d = self.artificial_delay.entry((from, to)).or_default();
        *d = delay;
    }

    pub fn set_delay(&mut self, delay: HashMap<(u8,u8), u64>) {
        self.artificial_delay = delay;
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
            let mut log = String::new();
            log += format!("Mined block {} ", &hex::encode(block.digest())[..4]).as_ref();
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
                            //let mut stdout = stdout();
                            //execute!(stdout, cursor::MoveTo(0,1), Clear(ClearType::CurrentLine), SetForegroundColor(Color::White), Print(format!("Delay from {} to {} is {}ms",block.miner,id,d))).unwrap();
                            std::thread::sleep(Duration::from_millis(d));
                            sender.send(block).unwrap();
                        }).unwrap();
                        log += format!("delay to {}: {} ms;\t", id, d).as_ref();
                    } else {
                        sender.send(block.clone()).unwrap();
                    };
                }
            }
            let mut log_ = self.log.write().unwrap();
            log_.clear();
            log_.push_str(log.as_ref());
        }
    }

    pub fn start(self) {
        std::thread::Builder::new().name(format!("network")).spawn(move || self.main_loop()).unwrap();
    }
}