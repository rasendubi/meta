use log::trace;

use meta_core::MetaCore;
use meta_store::{Field, Store};

use crate::bytecode::{Chunk, Instruction};
use crate::parser::{parse, Error as ParseError};
use crate::vm::{Error as VmError, Vm};

#[derive(Debug)]
pub enum Error {
    ParseError(Vec<ParseError>),
    InterpretError(VmError),
}

pub fn interpret(store: &Store, entry: &Field) -> Result<(), Error> {
    let core = MetaCore::new(store);
    let expr = parse(&core, entry)?;

    trace!("parsed: {:?}", expr);

    // TODO: compile
    let mut chunk = Chunk::new();
    chunk.write(&Instruction::Halt).unwrap();

    let mut vm = Vm::new(chunk);
    let () = vm.run()?;

    Ok(())
}

impl From<Vec<ParseError>> for Error {
    fn from(e: Vec<ParseError>) -> Self {
        Error::ParseError(e)
    }
}

impl From<VmError> for Error {
    fn from(e: VmError) -> Self {
        Error::InterpretError(e)
    }
}
