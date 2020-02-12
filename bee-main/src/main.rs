mod bee;
mod config;
mod constants;
mod errors;
mod screen;
mod state;

pub use crate::bee::Bee;
pub use crate::config::{Config, Host, Peer};

use bee_common::logger;

use async_std::task;

fn main() {
    logger::init(log::LevelFilter::Info);
    //screen::init();

    task::block_on(async {
        match Config::load().await {
            Err(e) => {
                logger::error(&e.to_string());

                task::block_on(async {
                    task::sleep(std::time::Duration::from_millis(10000)).await;
                });
            }
            Ok(config) => {
                logger::info("Loaded config.");

                let mut bee = Bee::from_config(config);

                assert!(bee.run().is_ok());

                bee.shutdown();
            }
        }
    });

    //screen::exit();
}
