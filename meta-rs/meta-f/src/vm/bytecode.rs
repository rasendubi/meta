use num_enum::{IntoPrimitive, UnsafeFromPrimitive};

pub(crate) use crate::vm::chunk::DataRef;
pub(crate) use crate::vm::machine::Reg;

#[derive(Debug, Eq, PartialEq, Copy, Clone, IntoPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub(crate) enum OpCode {
    // u64 = 8 bytes

    // 1B opcode
    Halt,
    // 1B opcode, 1B register
    HaltReg,
    // 1B opcode 3B reserved 4B value offset (data)
    HaltValue,

    // 1B opcode, 1B result reg, 2B reserved, 4B cells to allocate (constant)
    AllocConst,
    // 1B opcode, 1B result reg, 1B cells to allocate (reg)
    AllocReg,

    // 1B opcode, 1B addr reg, 2B offset (signed constant), 1B reg to store, 3B reserved
    StoreReg,
    // 1B opcode, 1B addr reg, 2B offset (signed constant), 4B value offset (data)
    StoreValue,

    // 1B opcode, 1B result reg, 1B addr reg, 1B reserved, 2B offset (signed constant), 2B reserved
    Load,

    // 1B opcode, 1B result reg, 2B reserved, 4B value offset (data)
    ConstantValue,

    // 1B opcode, 1B reg to switch on, 4B N=number of cases, 2B reserved | Nx8 offsets to jump to
    Switch,

    // 1B opcode, 1B reg to jump to, 6B reserved
    JumpReg,
    // 1B opcode, 3B reserved, 4B offset to jump to (signed)
    JumpConst,

    // 1B opcode, 1B result reg, 1B op reg, 1B reserved, 4B i32 offset
    Offset,
    // 1B opcode, 1B result reg, 1B op1 reg, 1B op2 reg
    Add,
    // 1B opcode, 1B result reg, 1B from reg
    Move,
    // 1B opcode, 1B from reg, 1B to reg
    Swap,
}

#[derive(Debug)]
pub(crate) enum Instruction {
    Halt,
    HaltReg {
        reg: Reg,
    },
    HaltValue {
        value: DataRef,
    },
    AllocConst {
        result: Reg,
        cells_to_allocate: u32,
    },
    AllocReg {
        result: Reg,
        cells_to_allocate: Reg,
    },
    StoreReg {
        addr: Reg,
        offset: i16,
        reg_to_store: Reg,
    },
    StoreValue {
        addr: Reg,
        offset: i16,
        value: DataRef,
    },
    Load {
        result: Reg,
        addr: Reg,
        offset: i16,
    },
    ConstantValue {
        result: Reg,
        value: DataRef,
    },
    Switch {
        reg: Reg,
        offsets: Vec<i64>,
    },
    JumpReg {
        reg: Reg,
    },
    JumpConst {
        offset: i32,
    },
    Offset {
        result: Reg,
        op1: Reg,
        offset: i32,
    },
    Add {
        result: Reg,
        op1: Reg,
        op2: Reg,
    },
    Move {
        result: Reg,
        from: Reg,
    },
    Swap {
        from: Reg,
        to: Reg,
    },
}

impl Instruction {
    pub fn write<W>(&self, w: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Instruction::Halt => {
                let instruction: u64 = OpCode::Halt as u64;
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::HaltReg { reg } => {
                let instruction: u64 = OpCode::HaltReg as u64 | ((reg.0 as u64) << 8);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::HaltValue { value } => {
                let instruction: u64 = OpCode::HaltValue as u64 | ((value.0 as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::AllocConst {
                result,
                cells_to_allocate,
            } => {
                let instruction: u64 = (OpCode::AllocConst as u64)
                    | ((result.0 as u64) << 8)
                    | ((*cells_to_allocate as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::AllocReg {
                result,
                cells_to_allocate,
            } => {
                let instruction: u64 = (OpCode::AllocReg as u64)
                    | ((result.0 as u64) << 8)
                    | ((cells_to_allocate.0 as u64) << 16);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::StoreReg {
                addr,
                offset,
                reg_to_store,
            } => {
                let instruction: u64 = (OpCode::StoreReg as u64)
                    | ((addr.0 as u64) << 8)
                    | ((*offset as u64) << 16)
                    | ((reg_to_store.0 as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::StoreValue {
                addr,
                offset,
                value,
            } => {
                let instruction: u64 = (OpCode::StoreValue as u64)
                    | ((addr.0 as u64) << 8)
                    | ((*offset as u64) << 16)
                    | ((value.0 as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::Load {
                result,
                addr,
                offset,
            } => {
                let instruction: u64 = (OpCode::Load as u64)
                    | ((result.0 as u64) << 8)
                    | ((addr.0 as u64) << 16)
                    | ((*offset as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::ConstantValue { result, value } => {
                let instruction: u64 = (OpCode::ConstantValue as u64)
                    | ((result.0 as u64) << 8)
                    | ((value.0 as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::Switch { reg, offsets } => {
                let instruction = (OpCode::Switch as u64)
                    | ((reg.0 as u64) << 8)
                    | ((offsets.len() as u64) << 16);
                w.write_all(&instruction.to_ne_bytes())?;
                for offset in offsets.iter() {
                    w.write_all(&offset.to_ne_bytes())?;
                }
                Ok(())
            }
            Instruction::JumpReg { reg } => {
                let instruction = (OpCode::JumpReg as u64) | ((reg.0 as u64) << 8);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::JumpConst { offset } => {
                let instruction = (OpCode::JumpConst as u64) | ((*offset as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::Offset {
                result,
                op1,
                offset,
            } => {
                let instruction = (OpCode::Offset as u64)
                    | ((result.0 as u64) << 8)
                    | ((op1.0 as u64) << 16)
                    | ((*offset as u64) << 32);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::Add { result, op1, op2 } => {
                let instruction = (OpCode::Add as u64)
                    | ((result.0 as u64) << 8)
                    | ((op1.0 as u64) << 16)
                    | ((op2.0 as u64) << 24);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::Move { result, from } => {
                let instruction =
                    (OpCode::Move as u64) | ((result.0 as u64) << 8) | ((from.0 as u64) << 16);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::Swap { from, to } => {
                let instruction: u64 =
                    (OpCode::Swap as u64) | ((from.0 as u64) << 8) | ((to.0 as u64) << 16);
                w.write_all(&instruction.to_ne_bytes())
            }
        }
    }

    pub fn read<R>(r: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        let mut instruction: [u8; 8] = [0; 8];
        r.read_exact(&mut instruction)?;
        let instruction = u64::from_ne_bytes(instruction);

        let opcode = unsafe { OpCode::from_unchecked(instruction as u8) };
        Ok(match opcode {
            OpCode::Halt => Instruction::Halt,
            OpCode::HaltReg => {
                let reg = Reg((instruction >> 8) as u8);
                Instruction::HaltReg { reg }
            }
            OpCode::HaltValue => {
                let value = DataRef((instruction >> 32) as u32);
                Instruction::HaltValue { value }
            }
            OpCode::AllocConst => {
                let result = Reg((instruction >> 8) as u8);
                let cells_to_allocate = (instruction >> 32) as u32;
                Instruction::AllocConst {
                    result,
                    cells_to_allocate,
                }
            }
            OpCode::AllocReg => {
                let result = Reg((instruction >> 8) as u8);
                let cells_to_allocate = Reg((instruction >> 16) as u8);
                Instruction::AllocReg {
                    result,
                    cells_to_allocate,
                }
            }
            OpCode::StoreReg => {
                let addr = Reg((instruction >> 8) as u8);
                let offset = (instruction >> 16) as i16;
                let reg_to_store = Reg((instruction >> 32) as u8);
                Instruction::StoreReg {
                    addr,
                    offset,
                    reg_to_store,
                }
            }
            OpCode::StoreValue => {
                let addr = Reg((instruction >> 8) as u8);
                let offset = (instruction >> 16) as i16;
                let value = DataRef((instruction >> 32) as u32);
                Instruction::StoreValue {
                    addr,
                    offset,
                    value,
                }
            }
            OpCode::Load => {
                let result = Reg((instruction >> 8) as u8);
                let addr = Reg((instruction >> 16) as u8);
                let offset = (instruction >> 32) as i16;
                Instruction::Load {
                    result,
                    addr,
                    offset,
                }
            }
            OpCode::ConstantValue => {
                let result = Reg((instruction >> 8) as u8);
                let value = DataRef((instruction >> 32) as u32);
                Instruction::ConstantValue { result, value }
            }
            OpCode::Switch => {
                let reg = Reg((instruction >> 8) as u8);
                let n_offsets = instruction >> 16;
                let mut offsets = Vec::new();

                for _ in 0..n_offsets {
                    let mut offset = [0; 8];
                    r.read_exact(&mut offset)?;
                    offsets.push(i64::from_ne_bytes(offset));
                }

                Instruction::Switch { reg, offsets }
            }
            OpCode::JumpReg => {
                let reg = Reg((instruction >> 8) as u8);
                Instruction::JumpReg { reg }
            }
            OpCode::JumpConst => {
                let offset = (instruction >> 32) as i32;
                Instruction::JumpConst { offset }
            }
            OpCode::Offset => {
                let result = Reg((instruction >> 8) as u8);
                let op1 = Reg((instruction >> 16) as u8);
                let offset = (instruction >> 32) as i32;
                Instruction::Offset {
                    result,
                    op1,
                    offset,
                }
            }
            OpCode::Add => {
                let result = Reg((instruction >> 8) as u8);
                let op1 = Reg((instruction >> 16) as u8);
                let op2 = Reg((instruction >> 24) as u8);
                Instruction::Add { result, op1, op2 }
            }
            OpCode::Move => {
                let result = Reg((instruction >> 8) as u8);
                let from = Reg((instruction >> 16) as u8);
                Instruction::Move { result, from }
            }
            OpCode::Swap => {
                let from = Reg((instruction >> 8) as u8);
                let to = Reg((instruction >> 16) as u8);
                Instruction::Swap { from, to }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_shr_sign() {
        let original: i64 = -1;
        let packed = (original as u64) << 16;
        let result = (packed as i64) >> 16;
        assert_eq!(original, result);
    }
}
