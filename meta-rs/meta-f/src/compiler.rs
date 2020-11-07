use std::rc::Rc;

use im::HashMap;

use crate::cps::Exp as CExp;
use crate::cps::*;
use crate::parser::{
    Binding, Constructor, Expr, Function, Identifier, RunTest, Statement, TypeDef,
};

pub(crate) fn entry_to_cps(gen: &mut VarGen, e: &RunTest) -> CExp {
    compile_expr(
        gen,
        Env::new(),
        &e.expr,
        Box::new(|_gen: &mut _, v| {
            CExp::Primop(Primop::Halt, Box::new([v]), Box::new([]), Box::new([]))
        }),
    )
}

#[derive(Debug, Clone)]
struct Env {
    variables: HashMap<Identifier, Value>,
    fields: HashMap<Identifier, /* offset: */ usize>, // constructors and constructor parameters
}

impl Env {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            fields: HashMap::new(),
        }
    }

    fn get_variable(&self, id: &Identifier) -> Option<&Value> {
        self.variables.get(id)
    }

    fn add_variable(&mut self, id: Identifier, value: Value) {
        self.variables.insert(id, value);
    }

    fn get_field(&self, id: &Identifier) -> Option<&usize> {
        self.fields.get(id)
    }

    fn add_field(&mut self, id: Identifier, offset: usize) {
        self.fields.insert(id, offset);
    }
}

fn compile_exprs<F>(gen: &mut VarGen, env: Env, es: &[Expr], and_then: F) -> CExp
where
    F: FnOnce(&mut VarGen, Vec<Value>) -> CExp,
{
    let x = es.iter().rfold(
        Box::new(and_then) as Box<dyn FnOnce(&mut VarGen, Vec<Value>) -> CExp>,
        |and_then, e| {
            let env = env.clone();
            Box::new(move |gen: &mut VarGen, mut vals: Vec<Value>| {
                compile_expr(gen, env, e, move |gen, v| {
                    vals.push(v);
                    and_then(gen, vals)
                })
            })
        },
    );
    x(gen, Vec::new())
}

fn compile_expr<'a, F>(gen: &'a mut VarGen, env: Env, e: &'a Expr, and_then: F) -> CExp
where
    F: FnOnce(&mut VarGen, Value) -> CExp,
{
    match e {
        Expr::NumberLiteral(i) => and_then(gen, Value::Int(*i)),
        Expr::StringLiteral(s) => and_then(gen, Value::String(s.clone())),
        Expr::Identifier(identifier) => {
            let val = env.get_variable(identifier).unwrap();
            and_then(gen, val.clone())
        }
        Expr::App(f, args) => {
            let k = gen.next();
            let kv = gen.next();

            let next = and_then(gen, Value::Var(kv));
            CExp::Fix(
                Box::new([FnDef(k, Box::new([kv]), Rc::new(next))]),
                Rc::new(compile_expr(
                    gen,
                    env.clone(),
                    f,
                    Box::new(move |gen: &mut _, f| {
                        compile_exprs(
                            gen,
                            env,
                            args,
                            Box::new(move |_gen: &mut VarGen, mut args: Vec<Value>| {
                                args.push(Value::Var(k));
                                CExp::App(f, args.into_boxed_slice())
                            })
                                as Box<dyn for<'r> FnOnce(&'r mut VarGen, Vec<Value>) -> CExp>,
                        )
                    }) as Box<dyn FnOnce(&mut VarGen, Value) -> CExp>,
                )),
            )
        }
        Expr::Function(f) => {
            let f_var = gen.next();
            let fndef = compile_fndef(gen, env, f, f_var);
            CExp::Fix(Box::new([fndef]), Rc::new(and_then(gen, Value::Var(f_var))))
        }
        Expr::Block(stmts) => {
            let k = gen.next();
            let v = gen.next();
            CExp::Fix(
                Box::new([FnDef(
                    k,
                    Box::new([v]),
                    Rc::new(and_then(gen, Value::Var(v))),
                )]),
                Rc::new(compile_block(
                    gen,
                    env,
                    stmts,
                    Box::new(|_gen: &mut _, res| CExp::App(Value::Var(k), Box::new([res])))
                        as Box<dyn FnOnce(&mut _, _) -> _>,
                )),
            )
        }
        Expr::TypeDef(TypeDef { constructors }) => {
            let fndefs = constructors
                .iter()
                .map(|c| compile_constructor(gen, c))
                .collect::<Box<[FnDef]>>();
            let vars = fndefs
                .iter()
                .map(|f| Value::Var(f.0))
                .collect::<Box<[Value]>>();

            let r = gen.next();

            CExp::Fix(
                fndefs,
                Rc::new(CExp::Record(vars, r, Rc::new(and_then(gen, Value::Var(r))))),
            )
        }
        Expr::Access(object, field) => compile_expr(
            gen,
            env.clone(),
            object,
            Box::new(move |gen: &mut VarGen, val: _| {
                let offset = env.get_field(field).expect("unable to get_field()");
                let r = gen.next();
                CExp::Select(
                    *offset as isize,
                    val,
                    r,
                    Rc::new(and_then(gen, Value::Var(r))),
                )
            }) as Box<dyn FnOnce(&mut VarGen, Value) -> CExp>,
        ),
    }
}

fn compile_block<F>(gen: &mut VarGen, env: Env, stmts: &[Statement], and_then: F) -> CExp
where
    F: FnOnce(&mut VarGen, Value) -> CExp,
{
    if stmts.is_empty() {
        return and_then(gen, Value::Int(0));
    }

    let stmt = &stmts[0];
    let rest = &stmts[1..];

    match stmt {
        Statement::Binding(binding) => {
            let Binding { identifier, value } = binding;
            match value {
                Expr::Function(f) => {
                    let f_var = gen.next();

                    let mut next_env = env;
                    next_env.add_variable(identifier.clone(), Value::Var(f_var));

                    let fndef = compile_fndef(gen, next_env.clone(), f, f_var);
                    CExp::Fix(
                        Box::new([fndef]),
                        Rc::new(compile_block(gen, next_env, rest, and_then)),
                    )
                }
                Expr::TypeDef(t) => {
                    let mut next_env = env.clone();
                    for (i, c) in t.constructors.iter().enumerate() {
                        next_env.add_field(c.identifier.clone(), i);

                        for (j, p) in c.parameters.iter().enumerate() {
                            next_env.add_field(p.id.clone(), j);
                        }
                    }

                    compile_expr(
                        gen,
                        env,
                        value,
                        Box::new(move |gen: &mut _, v| {
                            next_env.add_variable(identifier.clone(), v);
                            compile_block(gen, next_env, rest, and_then)
                        }) as Box<dyn FnOnce(&mut _, _) -> _>,
                    )
                }
                value => {
                    let mut next_env = env.clone();
                    compile_expr(
                        gen,
                        env,
                        value,
                        Box::new(move |gen: &mut _, v| {
                            next_env.add_variable(identifier.clone(), v);
                            compile_block(gen, next_env, rest, and_then)
                        }) as Box<dyn FnOnce(&mut _, _) -> _>,
                    )
                }
            }
        }
        Statement::Expr(expr) => {
            if rest.is_empty() {
                compile_expr(gen, env, expr, Box::new(and_then))
            } else {
                let and_then = Box::new(and_then) as Box<dyn FnOnce(&mut _, _) -> _>;
                compile_expr(
                    gen,
                    env.clone(),
                    expr,
                    Box::new(move |gen: &mut _, _: Value| compile_block(gen, env, rest, and_then))
                        as Box<dyn FnOnce(&mut _, _) -> _>,
                )
            }
        }
    }
}

fn compile_fndef(gen: &mut VarGen, env: Env, f: &Function, f_var: Var) -> FnDef {
    let Function { parameters, body } = f;

    let mut params = parameters.iter().map(|_| gen.next()).collect::<Vec<_>>();

    let k = gen.next(); // return continuation
    params.push(k);
    let params = params;

    let mut next_env = env;
    parameters.iter().zip(params.iter()).for_each(|(p, var)| {
        next_env.add_variable(p.id.clone(), Value::Var(*var));
    });

    FnDef(
        f_var,
        params.into_boxed_slice(),
        Rc::new(compile_expr(
            gen,
            next_env,
            body,
            Box::new(|_gen: &mut _, res| CExp::App(Value::Var(k), Box::new([res])))
                as Box<dyn FnOnce(&mut _, _) -> _>,
        )),
    )
}

fn compile_constructor(gen: &mut VarGen, constructor: &Constructor) -> FnDef {
    let Constructor {
        parameters,
        identifier: _,
    } = constructor;

    let var = gen.next();

    let parameters = parameters.iter().map(|_| gen.next()).collect::<Vec<_>>();

    let mut all_params = parameters.clone();
    let k = gen.next();
    all_params.push(k);

    let r = gen.next();

    let body = CExp::Record(
        parameters.iter().copied().map(Value::Var).collect(),
        r,
        Rc::new(CExp::App(Value::Var(k), Box::new([Value::Var(r)]))),
    );

    FnDef(var, all_params.into_boxed_slice(), Rc::new(body))
}
