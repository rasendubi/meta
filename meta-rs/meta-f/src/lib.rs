#![allow(dead_code)]

mod bytecode;
mod closure_conversion;
// mod compile;
mod cps;
// mod interpret;
mod cps_to_bytecode;
mod memory;
mod vm;

// pub use interpret::interpret;
// pub use interpret::Error;

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::io::Cursor;
//
//     static STORE: &str = include_str!("../../store.meta");
//
//     #[test]
//     fn test_42() -> std::io::Result<()> {
//         let store = serde_json::from_reader(Cursor::new(STORE))?;
//
//         let meta_f_test = "ckgrnl18v000cxama1mpves0c".into();
//
//         interpret(&store, &meta_f_test).unwrap();
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_hello_world() -> std::io::Result<()> {
//         let store = serde_json::from_reader(Cursor::new(STORE))?;
//
//         let meta_f_test = "ckgrnm5bt000ixamakqelhqwg".into();
//
//         interpret(&store, &meta_f_test).unwrap();
//
//         Ok(())
//     }
// }
