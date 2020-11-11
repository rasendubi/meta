use std::rc::Rc;

use log::{log_enabled, trace, Level};

use crate::parser::RunTest;
use crate::vm::chunk::Chunk;

use crate::compiler::closure_conversion::closure_conversion;
use crate::compiler::cps::VarGen;
use crate::compiler::cps_to_bytecode::cps_to_bytecode;
use crate::compiler::entry_to_cps::entry_to_cps;

pub(crate) fn compile(expr: &RunTest) -> Chunk {
    let mut gen = VarGen::new(0);
    trace!("parsed: {:?}", expr);

    let cps = Rc::new(entry_to_cps(&mut gen, expr));
    trace!("cps: {:?}", cps);

    let cps = closure_conversion(&mut gen, &cps);
    trace!("closure_converted: {:?}", cps);

    let chunk = cps_to_bytecode(&cps);
    if log_enabled!(Level::Trace) {
        chunk.disassemble(&mut std::io::stderr()).unwrap();
    }

    chunk
}
