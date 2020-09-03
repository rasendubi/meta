use std::fs::File;
use std::io::BufReader;

use meta_store::{MetaStore, Result};

fn main() -> Result<()> {
    let f = File::open("../core.meta")?;
    let store = MetaStore::from_reader(BufReader::new(&f))?;

    meta_editor::main(store).expect("Error running meta-editor");

    Ok(())
}
