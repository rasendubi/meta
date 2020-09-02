use std::fs::File;
use std::io::BufReader;

use meta_core::MetaCore;
use meta_editor::core_layout::core_layout_entities;
use meta_editor::layout::simple_doc_to_string;
use meta_store::{MetaStore, Result};

fn main() -> Result<()> {
    let f = File::open("../core.meta")?;
    let store = MetaStore::from_reader(BufReader::new(&f))?;
    let core = MetaCore::new(&store);
    let rich_doc = core_layout_entities(&core);
    let layout = meta_pretty::layout(&rich_doc, 80);
    let s = simple_doc_to_string(&layout);

    println!("{}", s);

    Ok(())
}
