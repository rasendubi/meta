use std::io::Cursor;

use crate::vm::bytecode::*;

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
    use crate::vm::value::*;

    #[test]
    fn disassemble() {
        let mut chunk = Chunk::new();

        [
            Instruction::ConstantValue {
                result: Reg(1),
                value: Value::number(1),
            },
            Instruction::ConstantValue {
                result: Reg(2),
                value: Value::number(1),
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
