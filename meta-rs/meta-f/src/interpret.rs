use meta_store::{Field, Store};

use crate::compile::{compile, Error as CompileError};
use crate::vm::{Error as VmError, Vm};

#[derive(Debug)]
pub enum Error {
    CompileError(CompileError),
    InterpretError(VmError),
}

pub fn interpret(store: &Store, entry: &Field) -> Result<(), Error> {
    let chunk = compile(store, entry)?;

    let mut vm = Vm::new(chunk);
    let () = vm.run()?;

    Ok(())
}

impl From<CompileError> for Error {
    fn from(e: CompileError) -> Self {
        Error::CompileError(e)
    }
}

impl From<VmError> for Error {
    fn from(e: VmError) -> Self {
        Error::InterpretError(e)
    }
}
