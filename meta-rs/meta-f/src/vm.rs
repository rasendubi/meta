use crate::bytecode::{Chunk, OpCode};
use crate::value::FValue;

#[derive(Debug)]
pub(crate) struct Vm {
    chunk: Chunk,
    ip: usize,
}

#[derive(Debug)]
pub enum Error {}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk, ip: 0 }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            let instruction = self.read_byte().into();
            match instruction {
                OpCode::Return => {
                    return Ok(());
                }

                OpCode::Constant => {
                    let constant = self.read_constant();
                    println!("{}", constant);
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let r = self.chunk.code(self.ip);
        self.ip += 1;
        r
    }

    fn read_constant(&mut self) -> &FValue {
        let constant_id = self.chunk.code(self.ip);
        self.ip += 1;
        self.chunk.constant(constant_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_return() {
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Return);
        let chunk = chunk;

        Vm::new(chunk).run().unwrap();
    }

    #[test]
    fn run_constant() {
        let mut chunk = Chunk::new();
        let constant = chunk
            .add_constant(FValue::String("Hello, world!".to_string()))
            .unwrap();
        chunk.write(OpCode::Constant);
        chunk.write(constant);
        chunk.write(OpCode::Return);
        let chunk = chunk;

        Vm::new(chunk).run().unwrap();
    }
}
