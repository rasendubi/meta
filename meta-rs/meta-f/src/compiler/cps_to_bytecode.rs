use std::io::{Cursor, Write};

use im::HashMap;

use crate::compiler::cps::*;
use crate::vm::bytecode::{Instruction, Reg};
use crate::vm::chunk::{Chunk, DataRef};
use crate::vm::value::Value as VmValue;

pub(crate) fn cps_to_bytecode(exp: &Exp) -> Chunk {
    let mut compilation = Compilation::new();
    compilation.compile(exp).unwrap();
    compilation.chunk
}

type RegisterAllocation = [Option<Var>; 256];

struct Compilation {
    chunk: Chunk,
    registers: RegisterAllocation,
    // TODO: we can deduplicate all constant values if we store
    // HashMap<Value, /* instructions: */ HashSet<usize>> first.
    to_patch: HashMap<usize, Var>,
    to_patch_data: HashMap<DataRef, Var>,
    functions: HashMap<Var, usize>,
}

impl Compilation {
    fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            registers: [None; 256],
            to_patch: HashMap::new(),
            to_patch_data: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn compile(&mut self, exp: &Exp) -> std::io::Result<()> {
        self.compile_exp(exp)?;
        self.patch()?;
        Ok(())
    }

    /// Returns the position where the function is located
    fn compile_fn(&mut self, f: &FnDef) -> std::io::Result<usize> {
        let FnDef(f, params, e) = f;

        let position = self.chunk.code().len();
        self.functions.insert(*f, position);

        self.registers = [None; 256];
        for (reg, param) in params.iter().enumerate() {
            self.registers[reg] = Some(*param);
        }

        self.compile_exp(e)?;

        Ok(position)
    }

    fn compile_exp(&mut self, exp: &Exp) -> std::io::Result<usize> {
        let position = self.chunk.code().len();
        match exp {
            Exp::Record(vals, var, e) => {
                let reg = self.register_for(*var);
                self.chunk.write(&Instruction::AllocConst {
                    result: reg,
                    cells_to_allocate: vals.len() as u32,
                })?;
                for (i, val) in vals.iter().enumerate() {
                    let offset = i as i16;
                    match val {
                        Value::Var(var) => {
                            let val_reg = self.register_of(*var);
                            self.chunk.write(&Instruction::StoreReg {
                                addr: reg,
                                offset,
                                reg_to_store: val_reg,
                            })?;
                        }
                        Value::Label(var) => {
                            let value = self.chunk.alloc_data(&[VmValue::invalid(var.0 as i32)]);
                            self.chunk.write(&Instruction::StoreValue {
                                addr: reg,
                                offset,
                                value,
                            })?;

                            self.to_patch_data.insert(value, *var);
                        }
                        Value::Int(v) => {
                            let value = self.chunk.alloc_data(&[VmValue::number(*v)]);
                            self.chunk.write(&Instruction::StoreValue {
                                addr: reg,
                                offset,
                                value,
                            })?;
                        }
                        Value::String(_) => {
                            todo!("Strings are not supported");
                        }
                        Value::ConstructorTag(var, n_cons) => {
                            let value = self
                                .chunk
                                .alloc_data(&[VmValue::constructor(var.0, *n_cons)]);
                            self.chunk.write(&Instruction::StoreValue {
                                addr: reg,
                                offset,
                                value,
                            })?;
                        }
                    }
                }
                self.compile_exp(e)?;
            }
            Exp::Select(i, val, var, e) => {
                if let Value::Var(ptr) = val {
                    let result = self.register_for(*var);
                    let reg = self.register_of(*ptr);
                    self.chunk.write(&Instruction::Load {
                        result,
                        addr: reg,
                        offset: *i as i16,
                    })?;
                } else {
                    panic!("Operator of select is not a variable");
                }
                self.compile_exp(e)?;
            }
            Exp::Offset(i, val, var, e) => {
                if let Value::Var(op1) = val {
                    let reg = self.register_for(*var);
                    let op1 = self.register_of(*op1);
                    self.chunk.write(&Instruction::Offset {
                        result: reg,
                        op1,
                        offset: *i as i32,
                    })?;
                } else {
                    panic!("Operator of offset is not a variable");
                }
                self.compile_exp(e)?;
            }

            Exp::App(f, vals) => {
                if let Value::Var(v) = f {
                    let reg = self.register_of(*v);

                    if (reg.0 as usize) < vals.len() {
                        // save address register from arguments

                        let target = Reg(vals.len() as u8);
                        if let Some(prev_var) = self.registers[target.0 as usize] {
                            self.chunk.write(&Instruction::Swap {
                                from: reg,
                                to: target,
                            })?;
                            self.registers[reg.0 as usize] = Some(prev_var);
                        } else {
                            self.chunk.write(&Instruction::Move {
                                result: target,
                                from: reg,
                            })?;
                            self.registers[reg.0 as usize] = None;
                        }

                        self.registers[target.0 as usize] = Some(*v);
                    }
                }

                for (reg, var) in vals.iter().enumerate().filter_map(|(i, val)| {
                    if let Value::Var(var) = val {
                        Some((Reg(i as u8), var))
                    } else {
                        None
                    }
                }) {
                    let reg_from = self.register_of(*var);
                    if reg_from == reg {
                        // Nothing to do. Variable is already in place.
                    } else if reg_from.0 < reg.0 {
                        // Same argument is passed multiple times. Copy it without invalidating the
                        // previous register.
                        if let Some(prev_var) = self.registers[reg.0 as usize] {
                            // if there was a previous var, stash it somewhere else.
                            let new_reg = self.register_for(prev_var);
                            self.chunk.write(&Instruction::Move {
                                from: reg,
                                result: new_reg,
                            })?;
                        }
                        self.chunk.write(&Instruction::Move {
                            from: reg_from,
                            result: reg,
                        })?;
                        self.registers[reg.0 as usize] = Some(*var);
                    } else {
                        // Swap/move register in place, forgetting the previous location.
                        if self.registers[reg.0 as usize].is_some() {
                            self.chunk.write(&Instruction::Swap {
                                from: reg_from,
                                to: reg,
                            })?;
                            self.registers.swap(reg_from.0 as usize, reg.0 as usize);
                        } else {
                            self.chunk.write(&Instruction::Move {
                                from: reg_from,
                                result: reg,
                            })?;
                            self.registers[reg_from.0 as usize] = None;
                            self.registers[reg.0 as usize] = Some(*var);
                        }
                    }
                }

                for (reg, val) in vals
                    .iter()
                    .enumerate()
                    .filter(|(_i, val)| !matches!(val, Value::Var(_)))
                {
                    let reg = Reg(reg as u8);
                    match val {
                        Value::Var(_) => panic!(),
                        Value::Label(label) => {
                            let value = self.chunk.alloc_data(&[VmValue::invalid(label.0 as i32)]);
                            self.chunk
                                .write(&Instruction::ConstantValue { result: reg, value })?;
                            self.to_patch_data.insert(value, *label);
                        }
                        Value::Int(i) => {
                            let value = self.chunk.alloc_data(&[VmValue::number(*i)]);
                            self.chunk
                                .write(&Instruction::ConstantValue { result: reg, value })?;
                        }
                        Value::String(_) => {
                            todo!("Strings are not supported");
                        }
                        Value::ConstructorTag(v, n_cons) => {
                            let value =
                                self.chunk.alloc_data(&[VmValue::constructor(v.0, *n_cons)]);
                            self.chunk
                                .write(&Instruction::ConstantValue { result: reg, value })?;
                        }
                    }
                }

                match f {
                    Value::Var(var) => {
                        let reg = self.register_of(*var);
                        self.chunk.write(&Instruction::JumpReg { reg })?;
                    }
                    Value::Label(var) => {
                        let pos = self.chunk.write(&Instruction::JumpConst {
                            offset: 100000 + var.0 as i32,
                        })?;
                        self.to_patch.insert(pos, *var);
                    }
                    _ => panic!("Invalid App target {:?}", f),
                }
            }
            Exp::Fix(fns, e) => {
                self.compile_exp(e)?;
                for f in fns.iter() {
                    self.compile_fn(f)?;
                }
            }
            Exp::Switch(val, es) => {
                if let Value::Var(var) = val {
                    let reg = self.register_of(*var);
                    let offsets = es.iter().map(|_| 0).collect();
                    let position = self.chunk.write(&Instruction::Switch { reg, offsets })?;

                    let real_positions = es
                        .iter()
                        .map(|e| self.compile_exp(e).unwrap())
                        .collect::<Vec<_>>();

                    let mut code = Cursor::new(self.chunk.code_mut());
                    code.set_position(position as u64 + 8);
                    for pos in real_positions.iter() {
                        let offset = pos - position;
                        code.write_all(&offset.to_ne_bytes())?;
                    }
                } else {
                    panic!("Unsupported value for switch: {:?}", val);
                }
            }
            Exp::Primop(op, ins, outs, es) => match (op, &**ins, &**outs, &**es) {
                (Primop::Halt, [], [], []) => {
                    self.chunk.write(&Instruction::Halt)?;
                }
                (Primop::Halt, [Value::Var(v)], [], []) => {
                    let reg = self.register_of(*v);
                    self.chunk.write(&Instruction::HaltReg { reg })?;
                }
                (Primop::Halt, [Value::Int(constant)], [], []) => {
                    let value = self.chunk.alloc_data(&[VmValue::number(*constant)]);
                    self.chunk.write(&Instruction::HaltValue { value })?;
                }
                (Primop::Plus, [Value::Var(op1), Value::Var(op2)], [res], [e]) => {
                    let result = self.register_for(*res);
                    self.chunk.write(&Instruction::Add {
                        result,
                        op1: self.register_of(*op1),
                        op2: self.register_of(*op2),
                    })?;
                    self.compile_exp(e)?;
                }
                (_, _, _, _) => panic!("wrong primop {:?}", exp),
            },
        }

        Ok(position)
    }

    fn patch(&mut self) -> std::io::Result<()> {
        let mut code = Cursor::new(self.chunk.code_mut());
        for (pos, var) in self.to_patch.iter() {
            let var_position = self.functions.get(var).unwrap();

            code.set_position(*pos as u64);
            let instruction = Instruction::read(&mut code)?;
            code.set_position(*pos as u64);
            match instruction {
                Instruction::JumpConst { offset: _ } => {
                    let offset = (*var_position as i32) - (*pos as i32);
                    Instruction::JumpConst { offset }
                }
                _ => panic!("Invalid instruction to patch: {:?}", instruction),
            }
            .write(&mut code)?;
        }

        for (data, var) in self.to_patch_data.iter() {
            let var_position = self.functions.get(var).unwrap();
            *self.chunk.data_mut(*data) = VmValue::number(*var_position as i32);
        }

        Ok(())
    }

    fn register_of(&self, var: Var) -> Reg {
        for (i, v) in self.registers.iter().enumerate() {
            if v == &Some(var) {
                return Reg(i as u8);
            }
        }
        panic!("Unable to find register of {:?}", var);
    }

    fn register_for(&mut self, var: Var) -> Reg {
        let free = self
            .registers
            .iter_mut()
            .enumerate()
            .find(|(_i, x)| x.is_none())
            .expect("Unable to find a free register");
        *free.1 = Some(var);
        Reg(free.0 as u8)
    }

    // TODO: implement proper register lifetime analysis
    // fn drop_variable(&mut self, var: Var) {
    //     todo!();
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::closure_conversion::closure_conversion;
    use std::rc::Rc;

    #[test]
    fn test_example() {
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

        chunk.disassemble(&mut std::io::stdout()).unwrap();
    }
}
