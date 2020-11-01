use im::{hashset, HashSet};

use meta_core::MetaCore;
use meta_store::Field;

#[derive(Debug, Clone)]
pub enum Error {
    UnexpectedType {
        entry: Field,
        expected: HashSet<Field>,
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
pub(crate) struct Identifier {
    entry: Field,
}

#[derive(Debug)]
pub(crate) struct Binding {
    identifier: Identifier,
    value: Expr,
}

#[derive(Debug)]
pub(crate) struct FunctionParameter {
    id: Identifier,
}

#[derive(Debug)]
pub(crate) struct Function {
    parameters: Vec<FunctionParameter>,
    body: Expr,
}

#[derive(Debug)]
pub(crate) enum Statement {
    Binding(Binding),
    Expr(Expr),
}

#[derive(Debug)]
pub(crate) enum Expr {
    NumberLiteral(i64),
    StringLiteral(String),
    Identifier(Identifier),
    App(Box<Expr>, Vec<Expr>),
    Function(Box<Function>),
    Block(Vec<Statement>),
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
            expected: hashset! {entry_point},
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
    let string_literal = "ckgkz6klf0000q2mas3dh1ms1".into();
    let function = "ckgvae1350000whmaqi356557".into();
    let application = "ckgxipqk50000c7mawkssuook".into();
    let block = "ckgz33mrp00005omaq226vzth".into();
    let identifier = "ckgz4197i000h9hmazilan75h".into();

    let type_ = core.meta_type(&entry).map(|d| &d.value);
    if type_ == Some(&number_literal) {
        let number_literal_value = "ckgkzbdt1000fq2maaedmj0rd".into();
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
        let string_literal_value = "ckgkz7deb0004q2maroxbccv8".into();
        let string =
            store
                .value(&entry, &string_literal_value)
                .ok_or_else(|| Error::ExpectedAttribute {
                    entry: entry.clone(),
                    attr: string_literal_value.clone(),
                })?;
        let value = string.value.to_string();

        Ok(Expr::StringLiteral(value))
    } else if type_ == Some(&identifier) {
        Ok(Expr::Identifier(parse_identifier(core, entry)?))
    } else if type_ == Some(&function) {
        let function_body = "ckgvag4va0004whmadyh1qnnv".into();
        let function_parameter = "ckgvahph5000bwhmaias0bwf7".into();

        let body = store
            .value(&entry, &function_body)
            .ok_or_else(|| Error::ExpectedAttribute {
                entry: entry.clone(),
                attr: function_body.clone(),
            })?;
        let body = parse_expr(core, &body.value)?;

        let param_entries = store
            .values(&entry, &function_parameter)
            .cloned()
            .unwrap_or_else(HashSet::new);

        let mut parameters = Vec::new();
        for param in param_entries
            .into_iter()
            .map(|p| parse_parameter(core, &p.value))
        {
            parameters.push(param?);
        }

        Ok(Expr::Function(Box::new(Function { parameters, body })))
    } else if type_ == Some(&application) {
        let application_fn = "ckgxiq1ot0004c7maalcx609z".into();
        let application_argument = "ckgxiqlw50009c7mask5ery0g".into();

        let f = store
            .value(entry, &application_fn)
            .ok_or_else(|| Error::ExpectedAttribute {
                entry: entry.clone(),
                attr: application_fn.clone(),
            })?;
        let f = parse_expr(core, &f.value)?;

        let arg_entries = store
            .values(entry, &application_argument)
            .cloned()
            .unwrap_or_else(HashSet::new);
        let arg_entries = core.order_datoms(arg_entries.iter());

        let mut args = Vec::new();
        for arg in arg_entries.into_iter().map(|e| parse_expr(core, &e.value)) {
            args.push(arg?);
        }

        Ok(Expr::App(Box::new(f), args))
    } else if type_ == Some(&block) {
        let block_statement = "ckgz33vst00045omakt15dloc".into();

        let stmt_entries = store
            .values(entry, &block_statement)
            .cloned()
            .unwrap_or_else(HashSet::new);
        let stmt_entries = core.order_datoms(stmt_entries.iter());

        let mut stmts = Vec::new();
        for stmt in stmt_entries
            .into_iter()
            .map(|e| parse_statement(core, &e.value))
        {
            stmts.push(stmt?);
        }

        Ok(Expr::Block(stmts))
    } else {
        Err(Error::UnexpectedType {
            entry: entry.clone(),
            expected: hashset! {number_literal, string_literal, identifier, function, application, block},
            actual: type_.cloned(),
        })
    }
}

fn parse_statement(core: &MetaCore, entry: &Field) -> Result<Statement, Error> {
    let store = &core.store;

    let number_literal = "ckgkz9xrn0009q2ma3hyzyejp".into();
    let string_literal = "ckgkz6klf0000q2mas3dh1ms1".into();
    let function = "ckgvae1350000whmaqi356557".into();
    let application = "ckgxipqk50000c7mawkssuook".into();
    let block = "ckgz33mrp00005omaq226vzth".into();
    let identifier = "ckgz4197i000h9hmazilan75h".into();
    let binding: Field = "ckgvali04000hwhmaw93ym25w".into();

    let allowed_types = hashset! {
        number_literal, // useless as a statement
        string_literal, // useless as a statement
        identifier, // useless as a statement
        function, // useless as a statement
        application,
        block,
        binding.clone(),
    };

    let unexpected_type = |actual| Error::UnexpectedType {
        entry: entry.clone(),
        expected: allowed_types.clone(),
        actual,
    };

    let type_ = core
        .meta_type(&entry)
        .map(|d| &d.value)
        .ok_or_else(|| unexpected_type(None))?;
    if !allowed_types.contains(type_) {
        return Err(unexpected_type(Some(type_.clone())));
    }

    if type_ == &binding {
        let binding_id = "ckgvaluy0000lwhmai73hadxb".into();
        let binding_value = "ckgvamn7n000rwhmaz95psjz9".into();

        let identifier =
            store
                .value(entry, &binding_id)
                .ok_or_else(|| Error::ExpectedAttribute {
                    entry: entry.clone(),
                    attr: binding_id,
                })?;
        let identifier = parse_identifier(core, &identifier.value)?;

        let value = store
            .value(entry, &binding_value)
            .ok_or_else(|| Error::ExpectedAttribute {
                entry: entry.clone(),
                attr: binding_value.clone(),
            })?;
        let value = parse_expr(core, &value.value)?;

        Ok(Statement::Binding(Binding { identifier, value }))
    } else {
        // must be some expression
        Ok(Statement::Expr(parse_expr(core, entry)?))
    }
}

fn parse_parameter(core: &MetaCore, param: &Field) -> Result<FunctionParameter, Error> {
    let parameter_identifier = "ckgz42xkx000s9hma2njbx3i7".into();
    let identifier = core
        .store
        .value(param, &parameter_identifier)
        .ok_or_else(|| Error::ExpectedAttribute {
            entry: param.clone(),
            attr: parameter_identifier,
        })?;

    Ok(FunctionParameter {
        id: parse_identifier(core, &identifier.value)?,
    })
}

fn parse_identifier(_core: &MetaCore, entry: &Field) -> Result<Identifier, Error> {
    Ok(Identifier {
        entry: entry.clone(),
    })
}
