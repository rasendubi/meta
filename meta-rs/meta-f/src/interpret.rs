use meta_core::MetaCore;
use meta_store::{Field, Store};

use crate::bytecode::{Chunk, Instruction};
use crate::parser::{parse, Error as ParseError};
use crate::vm::{Error as VmError, Vm};

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    InterpretError(VmError),
}

pub fn interpret(store: &Store, entry: &Field) -> Result<(), Error> {
    let core = MetaCore::new(store);
    let _expr = parse(&core, entry);

    // TODO: compile
    let mut chunk = Chunk::new();
    chunk.write(&Instruction::Halt).unwrap();

    let mut vm = Vm::new(chunk);
    let () = vm.run()?;

    Ok(())
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::ParseError(e)
    }
}

impl From<VmError> for Error {
    fn from(e: VmError) -> Self {
        Error::InterpretError(e)
    }
}
