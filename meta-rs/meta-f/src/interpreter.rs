use log::trace;

use meta_core::MetaCore;
use meta_store::{Field, Store};

use crate::compiler::compile;
use crate::parser::{parse, Error as ParseError};
use crate::vm::value::Value;
use crate::vm::{Error as VmError, Vm};

#[derive(Debug)]
pub enum Error {
    ParseError(Vec<ParseError>),
    RunError(VmError),
}

pub fn interpret(store: &Store, entry: &Field) -> Result<Option<Value>, Error> {
    let core = MetaCore::new(store);

    let expr = parse(&core, entry)?;
    trace!("parsed: {:?}", expr);

    let chunk = compile(&expr);

    let mut vm = Vm::new(chunk);
    Ok(vm.run()?)
}

impl From<Vec<ParseError>> for Error {
    fn from(e: Vec<ParseError>) -> Self {
        Error::ParseError(e)
    }
}

impl From<VmError> for Error {
    fn from(e: VmError) -> Self {
        Error::RunError(e)
    }
}
