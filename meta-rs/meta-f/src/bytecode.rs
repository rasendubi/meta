use std::{convert::TryInto, io::Write};

use crate::value::FValue;

#[derive(Debug)]
pub enum Error {
    TooMuchConstants,
}

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum OpCode {
    Return,
    Constant,
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
    constants: Vec<FValue>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn code(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    pub fn constant(&self, constant_id: u8) -> &FValue {
        &self.constants[constant_id as usize]
    }

    pub fn write<T: Into<u8>>(&mut self, byte: T) {
        self.code.push(byte.into());
    }

    pub fn add_constant(&mut self, value: FValue) -> Result<u8, Error> {
        let idx = self
            .constants
            .len()
            .try_into()
            .map_err(|_| Error::TooMuchConstants)?;
        self.constants.push(value);
        Ok(idx)
    }

    #[allow(dead_code)]
    pub fn disassemble<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut offset = 0;

        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset, w)?;
        }

        Ok(())
    }

    fn disassemble_instruction<W: Write>(
        &self,
        offset: usize,
        w: &mut W,
    ) -> std::io::Result<usize> {
        write!(w, "{:04} ", offset)?;

        let instruction = OpCode::from(self.code[offset]);
        match instruction {
            OpCode::Return => {
                writeln!(w, "{:?}", instruction)?;

                Ok(offset + 1)
            }
            OpCode::Constant => {
                let constant = self.code[offset + 1];
                writeln!(
                    w,
                    "{:?} {:?} ({:?})",
                    instruction, constant, self.constants[constant as usize]
                )?;

                Ok(offset + 2)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disassemble() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(FValue::Number(42)).unwrap();
        chunk.write(OpCode::Constant);
        chunk.write(constant);

        chunk.write(OpCode::Return);

        chunk.disassemble(&mut std::io::stdout()).unwrap();
    }
}
