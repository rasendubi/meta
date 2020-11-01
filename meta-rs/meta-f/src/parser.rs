use im::{hashset, HashSet};

use meta_core::MetaCore;
use meta_store::{Datom, Field};

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
pub(crate) struct EntryPoint {
    pub expr: Expr,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct Identifier {
    pub entry: Field,
}

#[derive(Debug)]
pub(crate) struct Binding {
    pub identifier: Identifier,
    pub value: Expr,
}

#[derive(Debug)]
pub(crate) struct FunctionParameter {
    pub id: Identifier,
}

#[derive(Debug)]
pub(crate) struct Function {
    pub parameters: Vec<FunctionParameter>,
    pub body: Expr,
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

pub(crate) fn parse(core: &MetaCore, entry: &Field) -> Result<EntryPoint, Vec<Error>> {
    Parser::new(core).parse(entry)
}

struct Parser<'a> {
    core: &'a MetaCore<'a>,
    errors: Vec<Error>,
}

impl<'a> Parser<'a> {
    fn new(core: &'a MetaCore<'a>) -> Self {
        Self {
            core,
            errors: Vec::new(),
        }
    }

    fn parse(mut self, entry: &Field) -> Result<EntryPoint, Vec<Error>> {
        self.parse_entry(entry).map_err(|_| self.errors)
    }

    fn report_error(&mut self, err: Error) {
        self.errors.push(err);
    }

    fn expect_type(&mut self, entry: &Field, types: &HashSet<Field>) -> Result<&Field, ()> {
        let type_ = self.core.meta_type(entry).map(|d| &d.value);
        if type_.map_or(false, |type_| types.contains(type_)) {
            Ok(type_.unwrap())
        } else {
            self.report_error(Error::UnexpectedType {
                entry: entry.clone(),
                expected: types.clone(),
                actual: type_.cloned(),
            });
            Err(())
        }
    }

    fn required_attribute(&mut self, entry: &Field, attr: &Field) -> Result<Field, ()> {
        self.core
            .store
            .value(entry, attr)
            .map(|d| d.value.clone())
            .ok_or_else(|| {
                self.report_error(Error::ExpectedAttribute {
                    entry: entry.clone(),
                    attr: attr.clone(),
                });
            })
    }

    fn values(&self, entry: &Field, attr: &Field) -> Vec<Datom> {
        if let Some(datoms) = self.core.store.values(entry, attr) {
            self.core
                .order_datoms(datoms.iter())
                .into_iter()
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    fn parse_entry(&mut self, entry: &Field) -> Result<EntryPoint, ()> {
        let entry_point = "ckgrnb2q20000xamazg71jcf6".into();
        let entry_point_expr = "ckgrnjxj30006xamalz6xvuk7".into();

        self.expect_type(entry, &hashset! {entry_point})?;

        let expr = self.required_attribute(&entry, &entry_point_expr)?;
        Ok(EntryPoint {
            expr: self.parse_expr(&expr)?,
        })
    }

    fn parse_expr(&mut self, entry: &Field) -> Result<Expr, ()> {
        let number_literal: Field = "ckgkz9xrn0009q2ma3hyzyejp".into();
        let string_literal: Field = "ckgkz6klf0000q2mas3dh1ms1".into();
        let function: Field = "ckgvae1350000whmaqi356557".into();
        let application: Field = "ckgxipqk50000c7mawkssuook".into();
        let block: Field = "ckgz33mrp00005omaq226vzth".into();
        let identifier: Field = "ckgz4197i000h9hmazilan75h".into();

        let type_ = self.expect_type(
            entry,
            &hashset! {
                number_literal.clone(),
                string_literal.clone(),
                identifier.clone(),
                function.clone(),
                application.clone(),
                block.clone(),
            },
        )?;
        if type_ == &number_literal {
            let number_literal_value = "ckgkzbdt1000fq2maaedmj0rd".into();
            let number = self.required_attribute(entry, &number_literal_value)?;
            // TODO: handle error
            let value = number.as_ref().parse().unwrap();

            Ok(Expr::NumberLiteral(value))
        } else if type_ == &string_literal {
            let string_literal_value = "ckgkz7deb0004q2maroxbccv8".into();
            let value = self
                .required_attribute(entry, &string_literal_value)?
                .to_string();

            Ok(Expr::StringLiteral(value))
        } else if type_ == &identifier {
            Ok(Expr::Identifier(self.parse_identifier(entry)?))
        } else if type_ == &function {
            let function_body = "ckgvag4va0004whmadyh1qnnv".into();
            let function_parameter = "ckgvahph5000bwhmaias0bwf7".into();

            let body = self.required_attribute(entry, &function_body)?;
            let body = self.parse_expr(&body)?;

            let param_entries = self.values(entry, &function_parameter);

            let mut parameters = Vec::new();
            for param in param_entries
                .into_iter()
                .map(|p| self.parse_parameter(&p.value))
            {
                parameters.push(param?);
            }

            Ok(Expr::Function(Box::new(Function { parameters, body })))
        } else if type_ == &application {
            let application_fn = "ckgxiq1ot0004c7maalcx609z".into();
            let application_argument = "ckgxiqlw50009c7mask5ery0g".into();

            let f = self.required_attribute(entry, &application_fn)?;
            let f = self.parse_expr(&f)?;

            let arg_entries = self.values(entry, &application_argument);

            let mut args = Vec::new();
            for arg in arg_entries.into_iter().map(|e| self.parse_expr(&e.value)) {
                args.push(arg?);
            }

            Ok(Expr::App(Box::new(f), args))
        } else if type_ == &block {
            let block_statement = "ckgz33vst00045omakt15dloc".into();

            let stmt_entries = self.values(entry, &block_statement);

            let mut stmts = Vec::new();
            for stmt in stmt_entries
                .into_iter()
                .map(|e| self.parse_statement(&e.value))
            {
                stmts.push(stmt?);
            }

            Ok(Expr::Block(stmts))
        } else {
            panic!("Type not covered: {:?}", type_);
        }
    }

    fn parse_statement(&mut self, entry: &Field) -> Result<Statement, ()> {
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

        let type_ = self.expect_type(entry, &allowed_types)?;
        if type_ == &binding {
            let binding_id = "ckgvaluy0000lwhmai73hadxb".into();
            let binding_value = "ckgvamn7n000rwhmaz95psjz9".into();

            let identifier = self.required_attribute(entry, &binding_id)?;
            let identifier = self.parse_identifier(&identifier)?;

            let value = self.required_attribute(entry, &binding_value)?;
            let value = self.parse_expr(&value)?;

            Ok(Statement::Binding(Binding { identifier, value }))
        } else {
            // must be some expression
            Ok(Statement::Expr(self.parse_expr(entry)?))
        }
    }

    fn parse_parameter(&mut self, param: &Field) -> Result<FunctionParameter, ()> {
        let parameter_identifier = "ckgz42xkx000s9hma2njbx3i7".into();
        let identifier = self.required_attribute(param, &parameter_identifier)?;

        Ok(FunctionParameter {
            id: self.parse_identifier(&identifier)?,
        })
    }

    fn parse_identifier(&mut self, entry: &Field) -> Result<Identifier, ()> {
        Ok(Identifier {
            entry: entry.clone(),
        })
    }
}
