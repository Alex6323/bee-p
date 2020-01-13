mod prototype;

use common::{CONFIG, DEBUG, ENV_VAR};

use crate::prototype::Prototype;

use std::env;

fn main() {
    env::set_var(ENV_VAR, DEBUG);

    let mut prototype = Prototype::from_config(CONFIG);

    assert!(prototype.run().is_ok());

    env::remove_var(ENV_VAR);
}
