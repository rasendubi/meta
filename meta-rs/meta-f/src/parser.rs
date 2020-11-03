use im::{hashset, HashSet};

use meta_core::MetaCore;
use meta_store::{Datom, Field};

use crate::ids::*;

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
        self.expect_type(entry, &hashset! {ENTRY_POINT.clone()})?;
        let expr = self.required_attribute(&entry, &ENTRY_POINT_EXPR)?;
        Ok(EntryPoint {
            expr: self.parse_expr(&expr)?,
        })
    }

    fn parse_expr(&mut self, entry: &Field) -> Result<Expr, ()> {
        let type_ = self.expect_type(
            entry,
            &hashset! {
                NUMBER_LITERAL.clone(),
                STRING_LITERAL.clone(),
                IDENTIFIER.clone(),
                FUNCTION.clone(),
                APPLICATION.clone(),
                BLOCK.clone(),
            },
        )?;
        if type_ == (&NUMBER_LITERAL as &Field) {
            let number = self.required_attribute(entry, &NUMBER_LITERAL_VALUE)?;
            // TODO: handle error
            let value = number.as_ref().parse().unwrap();

            Ok(Expr::NumberLiteral(value))
        } else if type_ == &STRING_LITERAL as &Field {
            let value = self
                .required_attribute(entry, &STRING_LITERAL_VALUE)?
                .to_string();

            Ok(Expr::StringLiteral(value))
        } else if type_ == &IDENTIFIER as &Field {
            Ok(Expr::Identifier(self.parse_identifier(entry)?))
        } else if type_ == &FUNCTION as &Field {
            let body = self.required_attribute(entry, &FUNCTION_BODY)?;
            let body = self.parse_expr(&body)?;

            let param_entries = self.values(entry, &FUNCTION_PARAMETER);

            let mut parameters = Vec::new();
            for param in param_entries
                .into_iter()
                .map(|p| self.parse_parameter(&p.value))
            {
                parameters.push(param?);
            }

            Ok(Expr::Function(Box::new(Function { parameters, body })))
        } else if type_ == &APPLICATION as &Field {
            let f = self.required_attribute(entry, &APPLICATION_FN)?;
            let f = self.parse_expr(&f)?;

            let arg_entries = self.values(entry, &APPLICATION_ARGUMENT);

            let mut args = Vec::new();
            for arg in arg_entries.into_iter().map(|e| self.parse_expr(&e.value)) {
                args.push(arg?);
            }

            Ok(Expr::App(Box::new(f), args))
        } else if type_ == &BLOCK as &Field {
            let stmt_entries = self.values(entry, &BLOCK_STATEMENT);

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
        let allowed_types = hashset! {
            NUMBER_LITERAL.clone(), // useless as a statement
            STRING_LITERAL.clone(), // useless as a statement
            IDENTIFIER.clone(), // useless as a statement
            FUNCTION.clone(), // useless as a statement
            APPLICATION.clone(),
            BLOCK.clone(),
            BINDING.clone(),
        };

        let type_ = self.expect_type(entry, &allowed_types)?;
        if type_ == &BINDING as &Field {
            let identifier = self.required_attribute(entry, &BINDING_IDENTIFIER)?;
            let identifier = self.parse_identifier(&identifier)?;

            let value = self.required_attribute(entry, &BINDING_VALUE)?;
            let value = self.parse_expr(&value)?;

            Ok(Statement::Binding(Binding { identifier, value }))
        } else {
            // must be some expression
            Ok(Statement::Expr(self.parse_expr(entry)?))
        }
    }

    fn parse_parameter(&mut self, param: &Field) -> Result<FunctionParameter, ()> {
        let identifier = self.required_attribute(param, &PARAMETER_IDENTIFIER)?;
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
