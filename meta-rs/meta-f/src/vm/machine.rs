use std::fmt::Debug;
use std::io::Cursor;
use std::ops::{Index, IndexMut};

use log::{log_enabled, trace, Level};

use crate::vm::bytecode::Instruction;
use crate::vm::chunk::Chunk;
use crate::vm::memory::Memory;
use crate::vm::value::*;

#[derive(Debug)]
pub enum Error {
    OutOfMemory,
}

pub(crate) struct Vm {
    chunk: Chunk,
    registers: Registers,
    memory: Memory,
}

impl Debug for Vm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vm")
            .field("chunk", &self.chunk)
            .field("registers", &self.registers)
            .finish()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Reg(pub u8);

struct Registers([Value; 256]);

impl Registers {
    pub fn new() -> Self {
        Self([Value::invalid(0); 256])
    }

    pub fn swap(&mut self, reg1: Reg, reg2: Reg) {
        self.0.swap(reg1.0 as usize, reg2.0 as usize);
    }
}

impl Index<Reg> for Registers {
    type Output = Value;

    fn index(&self, reg: Reg) -> &Self::Output {
        &self.0[reg.0 as usize]
    }
}

impl IndexMut<Reg> for Registers {
    fn index_mut(&mut self, reg: Reg) -> &mut Self::Output {
        &mut self.0[reg.0 as usize]
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.0.as_ref(), f)
    }
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            registers: Registers::new(),
            memory: Memory::new(1024 * 1024 / std::mem::size_of::<Value>()), // 1Mb
        }
    }

    pub fn run(&mut self) -> Result<Option<Value>, Error> {
        if log_enabled!(target: "vm", Level::Trace) {
            self.chunk.disassemble(&mut std::io::stderr()).unwrap();
        }

        let mut cursor = Cursor::new(self.chunk.code());
        loop {
            trace!(target: "vm::registers", "{:?}", &self.registers);
            let position = cursor.position();
            trace!(target: "vm", "Position: {}", position);
            let instruction = Instruction::read(&mut cursor).unwrap();
            trace!(target: "vm", "Instruction: {:?}", instruction);

            match instruction {
                Instruction::Halt => {
                    return Ok(None);
                }
                Instruction::HaltReg { reg } => {
                    return Ok(Some(self.registers[reg]));
                }
                Instruction::HaltValue { value } => {
                    return Ok(Some(value));
                }
                Instruction::AllocConst {
                    result,
                    cells_to_allocate,
                } => {
                    let ptr = self.memory.allocate_cells(cells_to_allocate as usize);
                    if ptr.is_null() {
                        return Err(Error::OutOfMemory);
                    }
                    self.registers[result] = Value::from_ptr(ptr);
                }
                Instruction::AllocReg {
                    result,
                    cells_to_allocate,
                } => {
                    let cells_to_allocate = self.registers[cells_to_allocate].as_number() as usize;
                    let ptr = self.memory.allocate_cells(cells_to_allocate);
                    if ptr.is_null() {
                        return Err(Error::OutOfMemory);
                    }
                    self.registers[result] = Value::from_ptr(ptr);
                }
                Instruction::StoreReg {
                    addr,
                    offset,
                    reg_to_store,
                } => unsafe {
                    let addr = self.registers[addr].as_ptr();
                    let addr = addr.offset(offset as isize);
                    *addr = self.registers[reg_to_store];
                },
                Instruction::StoreValue {
                    addr,
                    offset,
                    value,
                } => unsafe {
                    let addr = self.registers[addr].as_ptr();
                    let addr = addr.offset(offset as isize);
                    *addr = value;
                },
                Instruction::Load {
                    result,
                    addr,
                    offset,
                } => unsafe {
                    let addr = self.registers[addr].as_ptr();
                    let addr = addr.offset(offset as isize);
                    self.registers[result] = *addr;
                },
                Instruction::ConstantValue { result, value } => {
                    self.registers[result] = value;
                }
                Instruction::Switch { reg: _, offsets: _ } => {
                    todo!();
                }
                Instruction::JumpReg { reg } => {
                    let addr = self.registers[reg].as_number();
                    cursor.set_position(addr as u64);
                }
                Instruction::JumpConst { offset } => {
                    let addr = (position as i64) + offset;
                    cursor.set_position(addr as u64);
                }
                Instruction::Offset {
                    result,
                    op1,
                    offset,
                } => {
                    self.registers[result] = Value::from_ptr(unsafe {
                        self.registers[op1].as_ptr().offset(offset as isize)
                    })
                }
                Instruction::Add { result, op1, op2 } => {
                    // TODO: check they are actually numbers
                    self.registers[result] = Value::number(
                        self.registers[op1].as_number() + self.registers[op2].as_number(),
                    );
                }
                Instruction::Move { result, from } => {
                    self.registers[result] = self.registers[from];
                }
                Instruction::Swap { from, to } => {
                    self.registers.swap(from, to);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Value;
    use super::*;
    use crate::compiler::closure_conversion::closure_conversion;
    use crate::compiler::cps::*;
    use crate::compiler::cps_to_bytecode::cps_to_bytecode;
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
            Instruction::ConstantValue {
                result: Reg(0),
                value: Value::number(42),
            },
            Instruction::Halt,
        ]
        .iter()
        .for_each(|i| {
            chunk.write(i).unwrap();
        });

        let mut vm = Vm::new(chunk);
        vm.run().unwrap();

        assert_eq!(Value::number(42), vm.registers[Reg(0)]);
    }

    #[test]
    fn run_1_plus_2() {
        let mut chunk = Chunk::new();

        [
            Instruction::ConstantValue {
                result: Reg(1),
                value: Value::number(1),
            },
            Instruction::ConstantValue {
                result: Reg(2),
                value: Value::number(2),
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

        assert_eq!(Value::number(3), vm.registers[Reg(3)]);
    }

    #[test]
    fn run_complex() {
        use crate::compiler::cps::Value;

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

        let chunk = cps_to_bytecode(&result);

        let mut vm = Vm::new(chunk);
        vm.run().unwrap();
    }
}
