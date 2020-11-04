#![allow(dead_code)]

mod bytecode;
mod closure_conversion;
mod compiler;
mod cps;
mod cps_to_bytecode;
pub mod ids;
mod interpret;
mod memory;
mod parser;
mod vm;

pub use interpret::interpret;
pub use interpret::Error;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use test_env_log::test;

    static STORE: &str = include_str!("../../store.meta");

    #[test]
    fn test_42() -> std::io::Result<()> {
        let store = serde_json::from_reader(Cursor::new(STORE))?;

        let meta_f_test = "ckgrnl18v000cxama1mpves0c".into();

        let result = interpret(&store, &meta_f_test).unwrap();

        assert_eq!(Some(42), result);

        Ok(())
    }

    #[ignore] // strings are not currently supported
    #[test]
    fn test_hello_world() -> std::io::Result<()> {
        let store = serde_json::from_reader(Cursor::new(STORE))?;

        let meta_f_test = "ckgrnm5bt000ixamakqelhqwg".into();

        interpret(&store, &meta_f_test).unwrap();

        Ok(())
    }

    #[test]
    fn test_functions() -> std::io::Result<()> {
        let store = serde_json::from_reader(Cursor::new(STORE))?;

        let meta_f_test = "ckgzjkf8r0001cjmazi6gmb8x".into();

        let result = interpret(&store, &meta_f_test).unwrap();

        assert_eq!(Some(43), result);

        Ok(())
    }
}
