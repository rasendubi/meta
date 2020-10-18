use std::fs::File;
use std::io::BufReader;

use meta_store::Result;

fn main() -> Result<()> {
    env_logger::init();

    let f = File::open("store.meta")?;
    let store = serde_json::from_reader(BufReader::new(&f))?;

    meta_editor::main(store);

    Ok(())
}
