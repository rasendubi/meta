use std::{fmt::Debug, io::Cursor};

use crate::bytecode::{Chunk, Instruction, Reg};

pub(crate) struct Vm {
    chunk: Chunk,
    registers: [u64; 256],
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
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut cursor = Cursor::new(self.chunk.code());
        loop {
            let instruction = Instruction::read(&mut cursor).unwrap();

            match instruction {
                Instruction::Halt => {
                    break;
                }
                Instruction::AllocConst {
                    result: _,
                    cells_to_allocate: _,
                } => {}
                Instruction::AllocReg {
                    result: _,
                    cells_to_allocate: _,
                } => {}
                Instruction::Store {
                    addr: _,
                    offset: _,
                    reg_to_store: _,
                } => {}
                Instruction::StoreConst {
                    addr: _,
                    offset: _,
                    constant: _,
                } => {}
                Instruction::Load {
                    result: _,
                    addr: _,
                    offset: _,
                } => {}
                Instruction::Constant { result, constant } => {
                    self.registers[result.0 as usize] = constant;
                }
                Instruction::Switch { reg: _, offsets: _ } => {}
                Instruction::JumpReg { reg: _ } => {}
                Instruction::JumpConst { offset: _ } => {}
                Instruction::Add { result, op1, op2 } => {
                    self.registers[result.0 as usize] =
                        self.registers[op1.0 as usize] + self.registers[op2.0 as usize];
                }
                Instruction::Move { result: _, from: _ } => {}
                Instruction::Swap { from: _, to: _ } => {}
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
}
