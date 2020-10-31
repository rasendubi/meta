use std::{fmt::Debug, io::Cursor};

use log::{log_enabled, trace, Level};

use crate::bytecode::{Chunk, Instruction, Reg};
use crate::memory::Memory;

pub(crate) struct Vm {
    chunk: Chunk,
    registers: [u64; 256],
    memory: Memory,
}

impl Debug for Vm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vm")
            .field("chunk", &self.chunk)
            .field("registers", &self.registers.as_ref())
            .finish()
    }
}

#[derive(Debug)]
pub enum Error {}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            registers: [0; 256],
            memory: Memory::new(1024 * 1024), // 1Mb
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        if log_enabled!(target: "vm", Level::Trace) {
            self.chunk.disassemble(&mut std::io::stderr()).unwrap();
        }

        let mut cursor = Cursor::new(self.chunk.code());
        loop {
            trace!(target: "vm::registers", "{:?}", &self.registers.as_ref());
            let position = cursor.position();
            trace!(target: "vm", "Position: {}", position);
            let instruction = Instruction::read(&mut cursor).unwrap();
            trace!(target: "vm", "Instruction: {:?}", instruction);

            match instruction {
                Instruction::Halt => {
                    break;
                }
                Instruction::AllocConst {
                    result,
                    cells_to_allocate,
                } => {
                    let ptr = self.memory.allocate((cells_to_allocate * 8) as usize);
                    self.registers[result.0 as usize] = ptr as u64;
                }
                Instruction::AllocReg {
                    result,
                    cells_to_allocate,
                } => {
                    let cells_to_allocate = self.registers[cells_to_allocate.0 as usize];
                    let ptr = self.memory.allocate((cells_to_allocate * 8) as usize);
                    self.registers[result.0 as usize] = ptr as u64;
                }
                Instruction::Store {
                    addr,
                    offset,
                    reg_to_store,
                } => unsafe {
                    let addr = self.registers[addr.0 as usize] as *mut u64;
                    let addr = addr.offset(offset as isize);
                    *addr = self.registers[reg_to_store.0 as usize];
                },
                Instruction::StoreConst {
                    addr,
                    offset,
                    constant,
                } => unsafe {
                    let addr = self.registers[addr.0 as usize] as *mut u64;
                    let addr = addr.offset(offset as isize);
                    *addr = constant;
                },
                Instruction::Load {
                    result,
                    addr,
                    offset,
                } => unsafe {
                    let addr = self.registers[addr.0 as usize] as *const u64;
                    let addr = addr.offset(offset as isize);
                    self.registers[result.0 as usize] = *addr;
                },
                Instruction::Constant { result, constant } => {
                    self.registers[result.0 as usize] = constant;
                }
                Instruction::Switch { reg: _, offsets: _ } => {
                    todo!();
                }
                Instruction::JumpReg { reg } => {
                    let addr = self.registers[reg.0 as usize];
                    cursor.set_position(addr);
                }
                Instruction::JumpConst { offset } => {
                    let addr = (position as i64) + offset;
                    cursor.set_position(addr as u64);
                }
                Instruction::Add { result, op1, op2 } => {
                    self.registers[result.0 as usize] = ((self.registers[op1.0 as usize] as i64)
                        + (self.registers[op2.0 as usize] as i64))
                        as u64;
                }
                Instruction::Move { result, from } => {
                    self.registers[result.0 as usize] = self.registers[from.0 as usize];
                }
                Instruction::Swap { from, to } => {
                    self.registers.swap(from.0 as usize, to.0 as usize);
                }
            }
        }

        Ok(())
    }

    fn register(&self, reg: Reg) -> u64 {
        self.registers[reg.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{closure_conversion::closure_conversion, cps::*, cps_to_bytecode::compile};
    use std::rc::Rc;

    #[test]
    fn run_halt() {
        let mut chunk = Chunk::new();
        &[Instruction::Halt].iter().for_each(|i| {
            chunk.write(i).unwrap();
        });

        let mut vm = Vm::new(chunk);
        vm.run().unwrap();
    }

    #[test]
    fn run_constant() {
        let mut chunk = Chunk::new();
        &[
            Instruction::Constant {
                result: Reg(0),
                constant: 42,
            },
            Instruction::Halt,
        ]
        .iter()
        .for_each(|i| {
            chunk.write(i).unwrap();
        });

        let mut vm = Vm::new(chunk);
        vm.run().unwrap();

        assert_eq!(42, vm.register(Reg(0)));
    }

    #[test]
    fn run_1_plus_2() {
        let mut chunk = Chunk::new();

        [
            Instruction::Constant {
                result: Reg(1),
                constant: 1,
            },
            Instruction::Constant {
                result: Reg(2),
                constant: 2,
            },
            Instruction::Add {
                result: Reg(3),
                op1: Reg(1),
                op2: Reg(2),
            },
            Instruction::Halt,
        ]
        .iter()
        .for_each(|i| {
            chunk.write(i).unwrap();
        });

        let mut vm = Vm::new(chunk);
        vm.run().unwrap();

        assert_eq!(3, vm.register(Reg(3)));
    }

    #[test]
    fn run_complex() {
        let input = Rc::new(Exp::Fix(
            Box::new([
                // (define (f0 i1 k2) (k2 (+ i1 i1)))
                FnDef(
                    Var(0),
                    Box::new([Var(1), Var(2)]),
                    Rc::new(Exp::Primop(
                        Primop::Plus,
                        Box::new([Value::Var(Var(1)), Value::Var(Var(1))]),
                        Box::new([Var(3)]),
                        Box::new([Rc::new(Exp::App(
                            Value::Var(Var(2)),
                            Box::new([Value::Var(Var(3))]),
                        ))]),
                    )),
                ),
                // (define (f4 i5) (f0 i5 f6))
                FnDef(
                    Var(4),
                    Box::new([Var(5)]),
                    Rc::new(Exp::App(
                        Value::Var(Var(0)),
                        Box::new([Value::Var(Var(5)), Value::Var(Var(6))]),
                    )),
                ),
                // (define (f6 i7) (halt))
                FnDef(
                    Var(6),
                    Box::new([Var(7)]),
                    Rc::new(Exp::Primop(
                        Primop::Halt,
                        Box::new([]),
                        Box::new([]),
                        Box::new([]),
                    )),
                ),
            ]),
            // (f0 42 f4)
            Rc::new(Exp::App(
                Value::Var(Var(0)),
                Box::new([Value::Int(42), Value::Var(Var(4))]),
            )),
        ));
        let mut gen = VarGen { next_var: 9 };
        let result = closure_conversion(&mut gen, &input);

        let chunk = compile(&result);

        let mut vm = Vm::new(chunk);
        vm.run().unwrap();
    }
}
