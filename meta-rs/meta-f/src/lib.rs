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

    #[test]
    fn test_argument_passing() -> std::io::Result<()> {
        let store = serde_json::from_reader(Cursor::new(STORE))?;

        let meta_f_test = "ckh62qzu500009gma6hh890nj".into();

        let result = interpret(&store, &meta_f_test).unwrap();

        assert_eq!(Some(31), result);

        Ok(())
    }

    #[test]
    fn test_variable_binding() -> std::io::Result<()> {
        let store = serde_json::from_reader(Cursor::new(STORE))?;

        let meta_f_test = "ckh62t1xz0000e8magvtomp2n".into();

        let result = interpret(&store, &meta_f_test).unwrap();

        assert_eq!(Some(100), result);

        Ok(())
    }

    #[test]
    fn test_escaping_functions() -> std::io::Result<()> {
        let store = serde_json::from_reader(Cursor::new(STORE))?;

        let meta_f_test = "ckh62yayg0000x9maiza5ybg2".into();

        let result = interpret(&store, &meta_f_test).unwrap();

        assert_eq!(Some(15), result);

        Ok(())
    }
}
