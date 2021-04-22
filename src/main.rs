#[macro_use]
extern crate crossterm;

mod miner;
mod block;
mod block_tree;
mod network;

use crossterm::{cursor, Result};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType};
use crossterm::style::{Color, SetForegroundColor, SetBackgroundColor, Print};
use crossterm::event::{read, Event, KeyEvent};
use std::io::{stdout, Write};
use std::collections::HashMap;
use crate::miner::Miner;
use std::sync::mpsc::{channel, Sender};
use crate::network::Network;
use crate::block::Block;
use std::time::Duration;

const N: u8 = 4;


pub fn draw_block_hash(id: u8, block: &Block) -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, cursor::MoveTo(10*id as u16,3+block.number as u16), Print(format!("{}", &hex::encode(block.digest())[..4])))
}

fn main() -> Result<()> {
    let color_map = vec![Color::Green, Color::Blue, Color::Red, Color::Cyan, Color::Yellow];

    let genesis = block::Block::genesis();
    let mut stdout = stdout();
    //enable_raw_mode()?;
    queue!(stdout, Clear(ClearType::All))?;
    queue!(stdout, cursor::MoveTo(0,1), Print(format!("Genesis {}", &hex::encode(genesis.digest())[..4])))?;

    let mut stores = HashMap::new();
    let (sender, receiver) = channel();
    let mut senders: HashMap<u8, Sender<Block>> = Default::default();
    for id in 0..N {
        queue!(stdout, cursor::MoveTo(20*id as u16,2), SetForegroundColor(color_map[id as usize]), Print(format!("{}", id)))?;
        let (sender_2, receiver_2) = channel();
        senders.insert(id, sender_2);
        let (miner, store) = Miner::new(id, N, sender.clone(), receiver_2);
        stores.insert(id, store);
        miner.start();
    }
    stdout.flush()?;
    let mut network = Network {
        n: N,
        from_miners: receiver,
        senders,
        artificial_delay: Default::default(),
    };
    // set artificial delay
    network.set_delay(2,3,8000);
    network.set_delay(3,0,8000);
    network.set_delay(0,1,8000);
    queue!(stdout, cursor::MoveTo(0,0), Print("add artificial delay"))?;
    network.start();

    std::thread::Builder::new().name(format!("drawing")).spawn(move || {
        loop {
            for id in 0..N {
                let read = stores.get(&id).unwrap();
                let read = read.read().unwrap();
                let tip_number = read.tip.number;
                for level in 0..tip_number {
                    if let Some(blocks ) = read.number_block.get(&level) {
                        for (i,block) in blocks.iter().enumerate() {
                            queue!(stdout, cursor::MoveTo(20*id as u16 + 5*i as u16,3+level as u16), SetForegroundColor(color_map[block.miner as usize]), Print(format!("{}", &hex::encode(block.digest())[..4]))).unwrap();
                        }
                    }
                }
                stdout.flush().unwrap();
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }).unwrap();

    //let _event = read()?;

    //disable_raw_mode()?;
    loop {
        std::thread::park();
    }
    Ok(())
}
