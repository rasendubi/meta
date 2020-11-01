use std::rc::Rc;

use log::trace;

use meta_core::MetaCore;
use meta_store::{Field, Store};

use crate::closure_conversion::closure_conversion;
use crate::compiler::entry_to_cps;
use crate::cps::VarGen;
use crate::cps_to_bytecode::compile;
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

    let mut gen = VarGen::new(0);

    let cps = Rc::new(entry_to_cps(&mut gen, &expr));
    trace!("cps: {:?}", cps);

    let cps = closure_conversion(&mut gen, &cps);
    trace!("closure_converted: {:?}", cps);

    let chunk = compile(&cps);

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
