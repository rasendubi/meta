use meta_core::MetaCore;
use meta_store::{Field, Store};

use crate::bytecode::{Chunk, OpCode};
use crate::value::FValue;

#[derive(Debug)]
pub enum Error {}

pub(crate) fn compile(store: &Store, entry: &Field) -> Result<Chunk, Error> {
    let mut chunk = Chunk::new();
    let core = MetaCore::new(store);

    compile_node(&mut chunk, &core, entry.clone())?;

    chunk.write(OpCode::Return);

    Ok(chunk)
}

fn compile_node(chunk: &mut Chunk, core: &MetaCore, entry: Field) -> Result<(), Error> {
    let store = &core.store;
    let entry_point = "ckgrnb2q20000xamazg71jcf6".into();
    let entry_point_expr = "ckgrnjxj30006xamalz6xvuk7".into();
    let number_literal = "ckgkz9xrn0009q2ma3hyzyejp".into();
    let number_literal_value = "ckgkzbdt1000fq2maaedmj0rd".into();
    let string_literal = "ckgkz6klf0000q2mas3dh1ms1".into();
    let string_literal_value = "ckgkz7deb0004q2maroxbccv8".into();

    let type_ = &core.meta_type(&entry).unwrap().value;
    if type_ == &entry_point {
        let expr = &store.value(&entry, &entry_point_expr).unwrap().value;
        return compile_node(chunk, core, expr.clone());
    } else if type_ == &number_literal {
        let number = store.value(&entry, &number_literal_value).unwrap();
        let value = number.value.as_ref().parse().unwrap();
        let constant = chunk.add_constant(FValue::Number(value)).unwrap();

        chunk.write(OpCode::Constant);
        chunk.write(constant);
    } else if type_ == &string_literal {
        let string = store.value(&entry, &string_literal_value).unwrap();
        let value = string.value.to_string();
        let constant = chunk.add_constant(FValue::String(value)).unwrap();

        chunk.write(OpCode::Constant);
        chunk.write(constant);
    } else {
        panic!("unknown type: {:?}", type_);
    }

    Ok(())
}
