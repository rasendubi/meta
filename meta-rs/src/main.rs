use meta::store;

use std::fs::File;
use std::io::BufReader;

fn main() -> store::Result<()> {
    let f = File::open("../core.meta")?;
    let store = store::MetaStore::from_reader(BufReader::new(&f))?;
    println!("{:?}", store);

    Ok(())
}
