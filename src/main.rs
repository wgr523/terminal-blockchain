#[macro_use]
extern crate crossterm;
#[macro_use]
extern crate horrorshow;

mod miner;
mod block;
mod block_tree;
mod network;
mod server;

use crossterm::{cursor};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType, ScrollUp, size};
use crossterm::style::{Color, SetForegroundColor, Print};
use std::io::{stdout, Write, BufReader};
use std::collections::HashMap;
use crate::miner::Miner;
use std::sync::mpsc::{channel, Sender};
use crate::network::Network;
use crate::block::Block;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use crate::block_tree::BlockTree;
use std::str::FromStr;
use std::error::Error;
use std::fs::File;

const N: u8 = 6;


// pub fn draw_block_hash(id: u8, block: &Block) -> Result<()> {
//     let mut stdout = stdout();
//     execute!(stdout, cursor::MoveTo(10*id as u16,MARGIN+block.number as u16), Print(format!("{}", &hex::encode(block.digest())[..4])))
// }

fn start_simulation() -> Result<(Network, HashMap<u8, Arc<RwLock<BlockTree>>>), Box<dyn Error>> {
    let mut stores = HashMap::new();
    let (sender, receiver) = channel();
    let mut senders: HashMap<u8, Sender<Block>> = Default::default();
    for id in 0..N {
        let (sender_2, receiver_2) = channel();
        senders.insert(id, sender_2);
        let (miner, store) = Miner::new(id, N, sender.clone(), receiver_2);
        stores.insert(id, store);
        miner.start();
    }
    let log = Arc::new(RwLock::new(String::new()));
    let network = Network {
        n: N,
        from_miners: receiver,
        senders,
        artificial_delay: Default::default(),
        log: log.clone(),
    };
    Ok((network, stores))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut network, stores) = start_simulation()?;
    let addr = std::net::SocketAddr::from_str("127.0.0.1:3333").expect("Parse address error");
    // Read the JSON contents of the file as an instance of `delay`.
    let mut delay: HashMap<(u8,u8), u64> = {
        if let Ok(file) = File::open("delay.json") {
            let reader = BufReader::new(file);
            let pretty_delay: Vec<(u8,u8,u64)> = serde_json::from_reader(reader)?;
            pretty_delay.into_iter().map(|(i,j,k)|((i,j), k)).collect()
        } else {
            Default::default()
        }
    };
    // for i in 0..N {
    //     for j in 0..N {
    //         delay.insert((i,j), 11000);
    //     }
    // }
    server::Server::start(addr,stores, &delay);
    network.set_delay(delay);
    network.start();
    loop {
        std::thread::park();
    }
}
/*
fn _main() -> Result<()> {
    const MARGIN: u16 = 4;
    let mut color_map = vec![Color::Green, Color::Blue, Color::Red, Color::Cyan, Color::Yellow, Color::Magenta];
    for _ in 0..200 {
        color_map.push(Color::White);
    }

    let mut stdout = stdout();
    //enable_raw_mode()?;
    queue!(stdout, Clear(ClearType::All))?;
    //let genesis = block::Block::genesis();
    //queue!(stdout, cursor::MoveTo(0,1), Print(format!("Genesis {}", &hex::encode(genesis.digest())[..4])))?;

    let mut stores = HashMap::new();
    let (sender, receiver) = channel();
    let mut senders: HashMap<u8, Sender<Block>> = Default::default();
    for id in 0..N {
        queue!(stdout, cursor::MoveTo(20*id as u16,1), SetForegroundColor(color_map[id as usize]), Print(format!("{}", id)))?;
        let (sender_2, receiver_2) = channel();
        senders.insert(id, sender_2);
        let (miner, store) = Miner::new(id, N, sender.clone(), receiver_2);
        stores.insert(id, store);
        miner.start();
    }
    stdout.flush()?;
    let log = Arc::new(RwLock::new(String::new()));
    let mut network = Network {
        n: N,
        from_miners: receiver,
        senders,
        artificial_delay: Default::default(),
        log: log.clone(),
    };
    // set artificial delay
    //for i in 0..N {
    //    for j in 0..N {
    //        network.set_single_delay(i,j,21000);
    //    }
    //}
    network.set_single_delay(4,5,11000);
    network.set_single_delay(5,0,11000);
    network.set_single_delay(0,1,11000);
    //network.set_single_delay(2,3,21000);
    //network.set_single_delay(3,4,21000);
    //network.set_single_delay(4,5,21000);
    //network.set_single_delay(5,0,21000);
    //network.set_single_delay(0,1,21000);
    //network.set_single_delay(2,3,6500);
    //network.set_single_delay(2,0,6500);
    //network.set_single_delay(2,1,6500);
    //network.set_single_delay(3,0,6500);
    //network.set_single_delay(3,1,6500);
    //network.set_single_delay(3,2,6500);
    //network.set_single_delay(0,1,6500);
    //network.set_single_delay(0,2,6500);
    //network.set_single_delay(0,3,6500);
    //network.set_single_delay(4,5,6500);
    //network.set_single_delay(5,0,6500);
    queue!(stdout, cursor::MoveTo(0,0), Print("add artificial delay"))?;
    network.start();
    let (cols, rows) = size()?;
    std::thread::Builder::new().name(format!("drawing")).spawn(move || -> Result<()> {
        let mut cnt = 0u64;
        let start_time = Instant::now();
        loop {
            queue!(stdout, cursor::MoveTo(0,2), Clear(ClearType::FromCursorDown), SetForegroundColor(Color::White), Print(log.read().unwrap()))?;
            queue!(stdout, cursor::MoveTo(0,3), SetForegroundColor(Color::White), Print(format!("Running time:  {} s", Instant::now().duration_since(start_time).as_secs())))?;
            cnt += 1;
            let begin = {
                let read = stores.get(&0).unwrap();
                let read = read.read().unwrap();
                let tip_number = read.tip.number;
                let r = MARGIN+tip_number as u16;
                if r >= rows {
                    (r-rows+1) as u64
                } else {
                    0
                }
            };
            for id in 0..N {
                let read = stores.get(&id).unwrap();
                let read = read.read().unwrap();
                let tip_number = read.tip.number;
                for level in begin..=tip_number {
                    if let Some(blocks ) = read.number_block.get(&level) {
                        for (i,block) in blocks.iter().enumerate() {
                            let r = MARGIN+(level-begin) as u16;
                            queue!(stdout, cursor::MoveTo(20*id as u16 + 5*i as u16,r), SetForegroundColor(color_map[block.miner as usize]), Print(format!("{}", &hex::encode(block.digest())[..4])))?;
                        }
                    }
                }
                drop(read);
                stdout.flush()?;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }).unwrap();

    //let _event = read()?;

    //disable_raw_mode()?;
    loop {
        std::thread::park();
    }
    Ok(())
}
*/