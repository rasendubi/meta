use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum FValue {
    String(String),
    Number(i64),
}

impl Display for FValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FValue::String(s) => f.write_str(s)?,

            FValue::Number(i) => {
                write!(f, "{}", i)?;
            }
        }

        Ok(())
    }
}
