mod store;
mod meta;

use std::io::BufReader;
use std::fs::File;

fn main() -> store::Result<()> {
    let f = File::open("../core.meta")?;
    let store = store::MetaStore::from_reader(BufReader::new(&f))?;
    println!("{:?}", store);

    Ok(())
}
