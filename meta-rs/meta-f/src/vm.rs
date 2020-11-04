use std::fmt::Debug;
use std::io::Cursor;
use std::ops::{Index, IndexMut};

use log::{log_enabled, trace, Level};

use crate::bytecode::{Chunk, Instruction, Reg};
use crate::memory::Memory;

#[derive(Debug)]
pub enum Error {}

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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) struct Value(u64);
impl Value {
    pub fn from_u64(v: u64) -> Self {
        Self(v)
    }

    pub fn from_i64(v: i64) -> Self {
        Self(v as u64)
    }

    pub fn from_ptr(ptr: *mut Value) -> Self {
        Self(ptr as u64)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }

    pub fn as_ptr(&self) -> *mut Value {
        self.0 as *mut Value
    }
}

struct Registers([Value; 256]);

impl Registers {
    pub fn new() -> Self {
        Self([Value(0); 256])
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
                Instruction::HaltConst { constant } => {
                    return Ok(Some(Value::from_u64(constant)));
                }
                Instruction::AllocConst {
                    result,
                    cells_to_allocate,
                } => {
                    let ptr = self.memory.allocate_cells(cells_to_allocate);
                    self.registers[result] = Value::from_ptr(ptr);
                }
                Instruction::AllocReg {
                    result,
                    cells_to_allocate,
                } => {
                    let cells_to_allocate = self.registers[cells_to_allocate].as_u64();
                    let ptr = self.memory.allocate_cells(cells_to_allocate);
                    self.registers[result] = Value::from_ptr(ptr);
                }
                Instruction::Store {
                    addr,
                    offset,
                    reg_to_store,
                } => unsafe {
                    let addr = self.registers[addr].as_ptr();
                    let addr = addr.offset(offset as isize);
                    *addr = self.registers[reg_to_store];
                },
                Instruction::StoreConst {
                    addr,
                    offset,
                    constant,
                } => unsafe {
                    let addr = self.registers[addr].as_ptr();
                    let addr = addr.offset(offset as isize);
                    *addr = Value::from_u64(constant);
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
                Instruction::Constant { result, constant } => {
                    self.registers[result] = Value::from_u64(constant);
                }
                Instruction::Switch { reg: _, offsets: _ } => {
                    todo!();
                }
                Instruction::JumpReg { reg } => {
                    let addr = self.registers[reg].as_u64();
                    cursor.set_position(addr);
                }
                Instruction::JumpConst { offset } => {
                    let addr = (position as i64) + offset;
                    cursor.set_position(addr as u64);
                }
                Instruction::Add { result, op1, op2 } => {
                    self.registers[result] = Value::from_i64(
                        self.registers[op1].as_i64() + self.registers[op2].as_i64(),
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

    fn register(&self, reg: Reg) -> Value {
        self.registers[reg]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cps::Value;
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

        assert_eq!(Value(42), vm.register(Reg(0)));
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

        assert_eq!(Value(3), vm.register(Reg(3)));
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

    #[test]
    fn value_align_u64() {
        assert_eq!(std::mem::align_of::<Value>(), std::mem::align_of::<u64>());
    }

    #[test]
    fn value_align_f64() {
        assert_eq!(std::mem::align_of::<Value>(), std::mem::align_of::<f64>());
    }

    #[test]
    fn value_align_ptr() {
        assert_eq!(
            std::mem::align_of::<Value>(),
            std::mem::align_of::<*mut u64>()
        );
    }
}
