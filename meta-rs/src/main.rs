use std::fs::File;
use std::io::BufReader;

use meta_store::{Result, Store};

fn main() -> Result<()> {
    env_logger::init();

    let f = File::open("../core.meta")?;
    let store = Store::from_reader(BufReader::new(&f))?;

    meta_editor::main(store);

    Ok(())
}
