use bee_network::{Address, Url};

use super::config::Config;

use async_std::task::block_on;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pingpong", about = "bee-network example")]
pub struct Args {
    #[structopt(long)]
    pub bind: String,

    #[structopt(long)]
    pub peers: Vec<String>,

    #[structopt(long)]
    pub msg: String,
}

impl Args {
    pub fn make_config(&self) -> Config {
        let mut peers = vec![];
        for peer in &self.peers {
            peers.push(block_on(Url::from_url_str(&peer)).unwrap());
        }

        Config {
            host_addr: block_on(Address::from_addr_str(&self.bind.clone()[..])).unwrap(),
            peers,
        }
    }
}
