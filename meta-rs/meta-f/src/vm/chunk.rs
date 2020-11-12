use std::io::Cursor;

use crate::vm::bytecode::*;
use crate::vm::value::Value;

/// Index into data segment.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub(crate) struct DataRef(pub u32);

#[derive(Debug)]
pub(crate) struct Chunk {
    code: Vec<u8>,
    data: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn code_mut(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    pub fn data(&self, data_ref: DataRef) -> *const Value {
        &self.data[data_ref.0 as usize] as *const Value
    }

    pub fn data_mut(&mut self, data_ref: DataRef) -> &mut Value {
        &mut self.data[data_ref.0 as usize]
    }

    pub fn write(&mut self, instruction: &Instruction) -> std::io::Result<usize> {
        let position = self.code.len();
        instruction.write(&mut self.code)?;
        Ok(position)
    }

    pub fn alloc_data(&mut self, vals: &[Value]) -> DataRef {
        // TODO: prepend tag for garbage collector
        let cur = self.data.len() as u32;
        self.data.extend_from_slice(vals);
        DataRef(cur)
    }

    pub fn disassemble<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        writeln!(w, "Data:")?;
        for (i, value) in self.data.iter().enumerate() {
            writeln!(w, "{:04} {:?}", i, value)?;
        }

        writeln!(w, "Code:")?;
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
    use crate::vm::value::*;

    #[test]
    fn disassemble() {
        let mut chunk = Chunk::new();

        let number_1 = chunk.alloc_data(&[Value::number(1)]);
        let number_2 = chunk.alloc_data(&[Value::number(2)]);

        [
            Instruction::ConstantValue {
                result: Reg(1),
                value: number_1,
            },
            Instruction::ConstantValue {
                result: Reg(2),
                value: number_2,
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
