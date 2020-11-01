use meta_core::MetaCore;
use meta_store::Field;

#[derive(Debug, Clone)]
pub enum Error {
    UnexpectedType {
        entry: Field,
        expected: Vec<Field>,
        actual: Option<Field>,
    },
    ExpectedAttribute {
        entry: Field,
        attr: Field,
    },
}

#[derive(Debug)]
pub(crate) struct EntryPoint(Expr);

#[derive(Debug)]
pub(crate) enum Expr {
    NumberLiteral(i64),
    StringLiteral(String),
}

// TODO: report multiple errors at once
pub(crate) fn parse(core: &MetaCore, entry: &Field) -> Result<EntryPoint, Error> {
    let store = &core.store;

    let entry_point = "ckgrnb2q20000xamazg71jcf6".into();
    let entry_point_expr = "ckgrnjxj30006xamalz6xvuk7".into();

    let type_ = core.meta_type(entry).map(|d| &d.value);
    if type_ != Some(&entry_point) {
        return Err(Error::UnexpectedType {
            entry: entry.clone(),
            expected: vec![entry_point],
            actual: type_.cloned(),
        });
    }

    let expr = store
        .value(&entry, &entry_point_expr)
        .map(|d| &d.value)
        .ok_or_else(|| Error::ExpectedAttribute {
            entry: entry.clone(),
            attr: entry_point_expr,
        })?;
    Ok(EntryPoint(parse_expr(core, expr)?))
}

fn parse_expr(core: &MetaCore, entry: &Field) -> Result<Expr, Error> {
    let store = &core.store;

    let number_literal = "ckgkz9xrn0009q2ma3hyzyejp".into();
    let number_literal_value = "ckgkzbdt1000fq2maaedmj0rd".into();
    let string_literal = "ckgkz6klf0000q2mas3dh1ms1".into();
    let string_literal_value = "ckgkz7deb0004q2maroxbccv8".into();

    let type_ = core.meta_type(&entry).map(|d| &d.value);
    if type_ == Some(&number_literal) {
        let number =
            store
                .value(&entry, &number_literal_value)
                .ok_or_else(|| Error::ExpectedAttribute {
                    entry: entry.clone(),
                    attr: number_literal_value,
                })?;
        // TODO: handle error
        let value = number.value.as_ref().parse().unwrap();

        Ok(Expr::NumberLiteral(value))
    } else if type_ == Some(&string_literal) {
        let string =
            store
                .value(&entry, &string_literal_value)
                .ok_or_else(|| Error::ExpectedAttribute {
                    entry: entry.clone(),
                    attr: string_literal_value.clone(),
                })?;
        let value = string.value.to_string();

        Ok(Expr::StringLiteral(value))
    } else {
        Err(Error::UnexpectedType {
            entry: entry.clone(),
            expected: vec![number_literal, string_literal],
            actual: type_.cloned(),
        })
    }
}
