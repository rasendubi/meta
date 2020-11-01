use std::rc::Rc;

use im::HashMap;
use log::trace;

use crate::cps::*;

pub(crate) fn closure_conversion(gen: &mut VarGen, exp: &Rc<Exp>) -> Rc<Exp> {
    fn lift_functions(
        gen: &mut VarGen,
        lifted_fns: &mut Vec<FnDef>,
        wrapper_fns: &mut Vec<FnDef>,
        closure_formats: &mut HashMap<Var, Rc<[Value]>>,
        exp: &Exp,
    ) {
        match exp {
            Exp::Record(_vals, _var, e) => {
                lift_functions(gen, lifted_fns, wrapper_fns, closure_formats, e)
            }
            Exp::Select(_i, _val, _var, e) => {
                lift_functions(gen, lifted_fns, wrapper_fns, closure_formats, e)
            }
            Exp::Offset(_i, _val, _var, e) => {
                lift_functions(gen, lifted_fns, wrapper_fns, closure_formats, e)
            }
            Exp::App(_f, _vals) => {}
            Exp::Fix(fns, e) => {
                let fn_vars = fns.iter().map(|f| f.0).collect::<Vec<Var>>();
                let wrappers = fns.iter().map(|_| (gen.next())).collect::<Vec<Var>>();
                let fn_free_vars = free_variables_fndefs(fns);

                let mut closure_format = wrappers.clone();
                closure_format.extend(fn_free_vars);
                let closure_format = closure_format;

                let closure_build_format = closure_format
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(i, v)| {
                        if i < fns.len() {
                            Value::Label(v)
                        } else {
                            Value::Var(v)
                        }
                    })
                    .collect::<Rc<[Value]>>();

                let fn_to_wrapper = fn_vars
                    .iter()
                    .copied()
                    .zip(wrappers.iter().copied())
                    .collect::<HashMap<Var, Var>>();
                for (f, wrapped_f_var) in fns.iter().zip(wrappers) {
                    let (lifted_fn, extra_args) = build_lifted_fn(gen, f);
                    let wrapper_fn = build_wrapper_fn(
                        gen,
                        &closure_format,
                        &fn_to_wrapper,
                        wrapped_f_var,
                        f,
                        &lifted_fn,
                        &*extra_args,
                    );

                    lifted_fns.push(lifted_fn);
                    wrapper_fns.push(wrapper_fn);

                    closure_formats.insert(f.0, closure_build_format.clone());

                    lift_functions(gen, lifted_fns, wrapper_fns, closure_formats, &f.2);
                }

                lift_functions(gen, lifted_fns, wrapper_fns, closure_formats, e);
            }
            Exp::Switch(_val, es) => es
                .iter()
                .for_each(|e| lift_functions(gen, lifted_fns, wrapper_fns, closure_formats, e)),
            Exp::Primop(_op, _ins, _outs, es) => es
                .iter()
                .for_each(|e| lift_functions(gen, lifted_fns, wrapper_fns, closure_formats, e)),
        }
    }

    fn build_lifted_fn(gen: &mut VarGen, f: &FnDef) -> (FnDef, Box<[Var]>) {
        let FnDef(_f_var, params, e) = f;

        let mut free_vars = free_variables(e);
        for p in params.iter() {
            free_vars.remove(p);
        }
        let mut free_vars = free_vars.into_iter().collect::<Vec<_>>();
        free_vars.sort_unstable();
        let free_vars = free_vars.into_boxed_slice();

        let extra_vars = free_vars
            .iter()
            .map(|x| (*x, gen.next()))
            .collect::<Vec<(Var, Var)>>();

        let new_f_var = gen.next();
        let mut new_params = Vec::from(&**params);
        new_params.extend(extra_vars.iter().map(|(_old, v)| v));
        let new_params = new_params.into_boxed_slice();
        let new_e = alpha_convert(&extra_vars.iter().copied().collect(), e);

        let new_f = FnDef(new_f_var, new_params, new_e);

        (new_f, extra_vars.iter().map(|x| x.0).collect())
    }

    fn build_wrapper_fn(
        gen: &mut VarGen,
        closure_format: &[Var],
        fn_to_wrapper: &HashMap<Var, Var>,
        var: Var,
        f: &FnDef,
        lifted_fn: &FnDef,
        extra_args: &[Var],
    ) -> FnDef {
        let mut my_params = lifted_fn.1[..f.1.len()]
            .iter()
            .map(|_| gen.next())
            .collect::<Vec<_>>();
        let closure_var = gen.next();
        my_params.push(closure_var);
        let my_params = my_params.into_boxed_slice();

        let extra_args_vars = extra_args.iter().map(|_| gen.next()).collect::<Vec<_>>();

        let mut args = Vec::from(&my_params[..my_params.len() - 1]);
        args.extend(extra_args_vars.iter().copied());
        let args = args.into_iter().map(Value::Var).collect();

        let my_offset = closure_format
            .iter()
            .position(|v| *v == var)
            .expect("can't find own offset") as isize;
        let body = extra_args.iter().zip(extra_args_vars).rfold(
            Rc::new(Exp::App(Value::Label(lifted_fn.0), args)),
            |exp, (arg, arg_var)| {
                Rc::new(if let Some(w) = fn_to_wrapper.get(arg) {
                    let offset = closure_format
                        .iter()
                        .position(|v| v == w)
                        .unwrap_or_else(|| {
                            panic!(
                                "can't find function {:?} in closure {:?}",
                                w, closure_format
                            )
                        }) as isize
                        - my_offset;
                    Exp::Offset(offset, Value::Var(closure_var), arg_var, exp)
                } else {
                    let offset = closure_format
                        .iter()
                        .position(|v| v == arg)
                        .unwrap_or_else(|| {
                            panic!("can't find var {:?} in closure {:?}", arg, closure_format)
                        }) as isize
                        - my_offset;
                    Exp::Select(offset, Value::Var(closure_var), arg_var, exp)
                })
            },
        );

        FnDef(var, my_params, body)
    }

    fn patch_exp(
        gen: &mut VarGen,
        closure_formats: &HashMap<Var, Rc<[Value]>>,
        e: &Exp,
    ) -> Rc<Exp> {
        let mut patch = |e: &Rc<Exp>| patch_exp(gen, closure_formats, e);
        Rc::new(match e {
            Exp::Record(vals, var, e) => Exp::Record(vals.clone(), *var, patch(e)),
            Exp::Select(i, val, var, e) => Exp::Select(*i, val.clone(), *var, patch(e)),
            Exp::Offset(i, val, var, e) => Exp::Offset(*i, val.clone(), *var, patch(e)),
            Exp::App(var, vals) => {
                if matches!(var, Value::Var(_)) {
                    let v = gen.next();
                    let mut args = Vec::from(&**vals);
                    args.push(var.clone());
                    let args = args.into_boxed_slice();

                    Exp::Select(0, var.clone(), v, Rc::new(Exp::App(Value::Var(v), args)))
                } else {
                    Exp::App(var.clone(), vals.clone())
                }
            }
            Exp::Fix(fns, e) => {
                let next_e = patch(e);
                if let Some(f) = fns.first() {
                    let closure_format = closure_formats.get(&f.0).unwrap_or_else(|| {
                        panic!("can't find closure format for function {:?}", f.0)
                    });

                    let closure_var = gen.next();
                    Exp::Record(
                        closure_format.iter().cloned().collect(),
                        closure_var,
                        fns.iter().enumerate().rfold(next_e, |e, (i, f)| {
                            Rc::new(Exp::Offset(i as isize, Value::Var(closure_var), f.0, e))
                        }),
                    )
                } else {
                    return next_e;
                }
            }
            Exp::Switch(val, es) => Exp::Switch(val.clone(), es.iter().map(patch).collect()),
            Exp::Primop(op, ins, outs, es) => Exp::Primop(
                *op,
                ins.clone(),
                outs.clone(),
                es.iter().map(patch).collect(),
            ),
        })
    }

    let mut lifted_fns = Vec::new();
    let mut wrapper_fns = Vec::new();
    let mut closure_formats = HashMap::new();
    lift_functions(
        gen,
        &mut lifted_fns,
        &mut wrapper_fns,
        &mut closure_formats,
        exp,
    );

    let mut fns = lifted_fns
        .into_iter()
        .map(|FnDef(f, params, e)| FnDef(f, params, patch_exp(gen, &closure_formats, &e)))
        .collect::<Vec<_>>();
    // wrapper functions don't need patching as they always call known functions.
    fns.extend(wrapper_fns);

    Rc::new(Exp::Fix(
        fns.into_boxed_slice(),
        patch_exp(gen, &closure_formats, &exp),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure_convertion() {
        let input = Rc::new(Exp::Fix(
            Box::new([FnDef(
                Var(0),
                Box::new([]),
                Rc::new(Exp::App(Value::Var(Var(1)), Box::new([]))),
            )]),
            Rc::new(Exp::Record(
                Box::new([Value::Var(Var(0))]),
                Var(2),
                Rc::new(Exp::App(Value::Var(Var(3)), Box::new([]))),
            )),
        ));
        let mut gen = VarGen { next_var: 4 };
        let result = closure_conversion(&mut gen, &input);

        println!("test_closure_convertion:\n{:#?}", result);
    }

    #[test]
    fn test_closure_mutually_recursive() {
        let input = Rc::new(Exp::Fix(
            Box::new([
                // (define (f0 i1 k2) (k2 (+ i1 1)))
                FnDef(
                    Var(0),
                    Box::new([Var(1), Var(2)]),
                    Rc::new(Exp::Primop(
                        Primop::Plus,
                        Box::new([Value::Var(Var(1)), Value::Int(1)]),
                        Box::new([Var(3)]),
                        Box::new([Rc::new(Exp::App(
                            Value::Var(Var(2)),
                            Box::new([Value::Var(Var(3))]),
                        ))]),
                    )),
                ),
                // (define (f4 i5) (f0 i5 f6))
                FnDef(
                    Var(4),
                    Box::new([Var(5)]),
                    Rc::new(Exp::App(
                        Value::Var(Var(0)),
                        Box::new([Value::Var(Var(5)), Value::Var(Var(6))]),
                    )),
                ),
                // (define (f6 i7) (f8 i7))
                FnDef(
                    Var(6),
                    Box::new([Var(7)]),
                    Rc::new(Exp::App(Value::Var(Var(8)), Box::new([Value::Var(Var(7))]))),
                ),
            ]),
            // (f0 42 f4)
            Rc::new(Exp::App(
                Value::Var(Var(0)),
                Box::new([Value::Int(42), Value::Var(Var(4))]),
            )),
        ));
        let mut gen = VarGen { next_var: 9 };
        let result = closure_conversion(&mut gen, &input);

        println!("test_closure_mutually_recursive:\n{:#?}", result);
    }
}
