use std::io::Cursor;

#[derive(Debug)]
pub enum Error {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Reg(pub u8);

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum OpCode {
    // u64 = 8 bytes

    // 1B opcode
    Halt,
    // 1B opcode, 1B register
    HaltReg,
    // 1B opcode | 8B const
    HaltConst,

    // 1B opcode, 1B result reg, 6B cells to allocate (constant)
    AllocConst,
    // 1B opcode, 1B result reg, 1B cells to allocate (reg)
    AllocReg,

    // 1B opcode, 1B addr reg, 4B offset (signed constant), 1B reg to store, 1B reserved
    Store,
    // 1B opcode, 1B addr reg, 4B offset (signed constant), 2B reserved | 8B constant to store
    StoreConst,

    // 1B opcode, 1B result reg, 4B offset (signed constant), 1B reserved
    Load,

    // 1B opcode, 1B result reg, 6B constant
    Constant,
    // 1B opcode, 1B result reg, 6B reserved | 8B constant
    ConstantBig,

    // 1B opcode, 1B reg to switch on, 4B N=number of cases, 2B reserved | Nx8 offsets to jump to
    Switch,

    // 1B opcode, 1B reg to jump to
    JumpReg,
    // 1B opcode, 7B relative offset to jump to
    JumpConst,

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
    HaltConst {
        constant: u64,
    },
    AllocConst {
        result: Reg,
        cells_to_allocate: u64,
    },
    AllocReg {
        result: Reg,
        cells_to_allocate: Reg,
    },
    Store {
        addr: Reg,
        offset: i32,
        reg_to_store: Reg,
    },
    StoreConst {
        addr: Reg,
        offset: i32,
        constant: u64,
    },
    Load {
        result: Reg,
        addr: Reg,
        offset: i32,
    },
    Constant {
        result: Reg,
        constant: u64,
    },
    Switch {
        reg: Reg,
        offsets: Vec<i64>,
    },
    JumpReg {
        reg: Reg,
    },
    JumpConst {
        offset: i64,
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
            Instruction::HaltConst { constant } => {
                let instruction: u64 = OpCode::HaltConst as u64;
                w.write_all(&instruction.to_ne_bytes())?;
                w.write_all(&constant.to_ne_bytes())
            }
            Instruction::AllocConst {
                result,
                cells_to_allocate,
            } => {
                let instruction: u64 = (OpCode::AllocConst as u64)
                    | ((result.0 as u64) << 8)
                    | (cells_to_allocate << 16);
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
            Instruction::Store {
                addr,
                offset,
                reg_to_store,
            } => {
                let instruction: u64 = (OpCode::Store as u64)
                    | ((addr.0 as u64) << 8)
                    | ((*offset as u32 as u64) << (2 * 8))
                    | ((reg_to_store.0 as u64) << (6 * 8));
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::StoreConst {
                addr,
                offset,
                constant,
            } => {
                let instruction: u64 = (OpCode::StoreConst as u64)
                    | ((addr.0 as u64) << 8)
                    | ((*offset as u32 as u64) << (2 * 8));
                w.write_all(&instruction.to_ne_bytes())?;
                w.write_all(&constant.to_ne_bytes())
            }
            Instruction::Load {
                result,
                addr,
                offset,
            } => {
                let instruction: u64 = (OpCode::Load as u64)
                    | ((result.0 as u64) << 8)
                    | ((addr.0 as u64) << 16)
                    | ((*offset as u32 as u64) << 24);
                w.write_all(&instruction.to_ne_bytes())
            }
            Instruction::Constant { result, constant } => {
                if *constant > 0xffff_ffff_ffff {
                    let instruction: u64 = (OpCode::ConstantBig as u64) | ((result.0 as u64) << 8);
                    w.write_all(&instruction.to_ne_bytes())?;
                    w.write_all(&constant.to_ne_bytes())
                } else {
                    let instruction: u64 =
                        (OpCode::Constant as u64) | ((result.0 as u64) << 8) | (constant << 16);
                    w.write_all(&instruction.to_ne_bytes())
                }
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
                let instruction = (OpCode::JumpConst as u64) | ((*offset as u64) << 8);
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

        let opcode: OpCode = (instruction as u8).into();
        Ok(match opcode {
            OpCode::Halt => Instruction::Halt,
            OpCode::HaltReg => {
                let reg = Reg((instruction >> 8) as u8);
                Instruction::HaltReg { reg }
            }
            OpCode::HaltConst => {
                let mut constant = [0; 8];
                r.read_exact(&mut constant)?;
                let constant = u64::from_ne_bytes(constant);

                Instruction::HaltConst { constant }
            }
            OpCode::AllocConst => {
                let result = Reg((instruction >> 8) as u8);
                let cells_to_allocate = instruction >> 16;
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
            OpCode::Store => {
                let addr = Reg((instruction >> 8) as u8);
                let offset = (instruction >> 16) as u32 as i32;
                let reg_to_store = Reg((instruction >> (6 * 8)) as u8);
                Instruction::Store {
                    addr,
                    offset,
                    reg_to_store,
                }
            }
            OpCode::StoreConst => {
                let addr = Reg((instruction >> 8) as u8);
                let offset = (instruction >> 16) as u32 as i32;

                let mut constant = [0; 8];
                r.read_exact(&mut constant)?;
                let constant = u64::from_ne_bytes(constant);

                Instruction::StoreConst {
                    addr,
                    offset,
                    constant,
                }
            }
            OpCode::Load => {
                let result = Reg((instruction >> 8) as u8);
                let addr = Reg((instruction >> 16) as u8);
                let offset = (instruction >> 24) as u32 as i32;
                Instruction::Load {
                    result,
                    addr,
                    offset,
                }
            }
            OpCode::Constant => {
                let result = Reg((instruction >> 8) as u8);
                let constant = instruction >> 16;
                Instruction::Constant { result, constant }
            }
            OpCode::ConstantBig => {
                let result = Reg((instruction >> 8) as u8);

                let mut constant = [0; 8];
                r.read_exact(&mut constant)?;
                let constant = u64::from_ne_bytes(constant);

                Instruction::Constant { result, constant }
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
                let offset = (instruction as i64) >> 8;
                Instruction::JumpConst { offset }
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

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        unsafe { std::mem::transmute(code) }
    }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        unsafe { std::mem::transmute(byte) }
    }
}

#[derive(Debug)]
pub(crate) struct Chunk {
    code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn code_mut(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    pub fn write(&mut self, instruction: &Instruction) -> std::io::Result<usize> {
        let position = self.code.len();
        instruction.write(&mut self.code)?;
        Ok(position)
    }

    pub fn disassemble<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut cursor = Cursor::new(&self.code);
        loop {
            let position = cursor.position();
            if let Ok(instruction) = Instruction::read(&mut cursor) {
                writeln!(w, "{:04} {:?}", position, instruction)?;
            } else {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shr_sign() {
        let original: i64 = -1;
        let packed = (original as u64) << 16;
        let result = (packed as i64) >> 16;
        assert_eq!(original, result);
    }

    #[test]
    fn disassemble() {
        let mut chunk = Chunk::new();

        [
            Instruction::Constant {
                result: Reg(1),
                constant: 1,
            },
            Instruction::Constant {
                result: Reg(2),
                constant: 1,
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

        chunk.disassemble(&mut std::io::stdout()).unwrap();
    }
}
