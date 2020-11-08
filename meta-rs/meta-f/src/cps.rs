use std::rc::Rc;

use im::{HashMap, HashSet};

// TODO: add name for debugging purposes
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub(crate) struct Var(pub u64);

pub(crate) struct VarGen {
    pub next_var: u64,
}

impl VarGen {
    pub fn new(next_var: u64) -> Self {
        Self { next_var }
    }

    pub fn next(&mut self) -> Var {
        let result = Var(self.next_var);
        self.next_var += 1;
        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) enum Value {
    Var(Var),
    Label(Var),
    Int(i32),
    String(String),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub(crate) enum Primop {
    Halt,
    #[allow(dead_code)]
    Plus,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct FnDef(pub Var, pub Box<[Var]>, pub Rc<Exp>);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) enum Exp {
    Record(Box<[Value]>, Var, Rc<Exp>),
    Select(isize, Value, Var, Rc<Exp>),
    Offset(isize, Value, Var, Rc<Exp>),
    App(Value, Box<[Value]>),
    Fix(Box<[FnDef]>, Rc<Exp>),
    #[allow(dead_code)]
    Switch(Value, Box<[Rc<Exp>]>),
    Primop(Primop, Box<[Value]>, Box<[Var]>, Box<[Rc<Exp>]>),
}

impl Value {
    pub fn as_var(&self) -> Option<Var> {
        if let Value::Var(var) = self {
            Some(*var)
        } else {
            None
        }
    }
}

pub(crate) fn free_variables_fndef(f: &FnDef) -> HashSet<Var> {
    let FnDef(_f, params, e) = f;
    let mut e_vars = free_variables(e);
    for param in params.iter() {
        e_vars.remove(param);
    }
    e_vars
}

pub(crate) fn free_variables_fndefs(fns: &[FnDef]) -> HashSet<Var> {
    let mut free_vars = HashSet::unions(fns.iter().map(|f| free_variables_fndef(f)));
    for FnDef(f, _params, _e) in fns.iter() {
        free_vars.remove(f);
    }
    free_vars
}

pub(crate) fn free_variables(exp: &Exp) -> HashSet<Var> {
    fn val_to_var(val: &Value) -> HashSet<Var> {
        val.as_var().map_or_else(HashSet::new, HashSet::unit)
    }
    fn vals_to_vars(vals: &[Value]) -> HashSet<Var> {
        HashSet::unions(vals.iter().map(val_to_var))
    }
    fn es_to_vars(es: &[Rc<Exp>]) -> HashSet<Var> {
        HashSet::unions(es.iter().map(|x| free_variables(x)))
    }

    match exp {
        Exp::Record(vals, var, e) => vals_to_vars(vals) + free_variables(e).without(var),
        Exp::Select(_i, val, var, e) => val_to_var(val) + free_variables(e).without(var),
        Exp::Offset(_i, val, var, e) => val_to_var(val) + free_variables(e).without(var),
        Exp::App(f, vals) => vals_to_vars(vals) + val_to_var(f),
        Exp::Switch(val, es) => val_to_var(val) + es_to_vars(es),
        Exp::Fix(fns, e) => {
            let fn_exp_vars = HashSet::unions(fns.iter().map(free_variables_fndef));

            let e_vars = free_variables(e);

            let mut result = e_vars + fn_exp_vars;
            for var in fns.iter().map(|x| x.0) {
                result.remove(&var);
            }
            result
        }
        Exp::Primop(_op, ins, outs, es) => {
            let mut es_vars = es_to_vars(es);
            for out in outs.iter() {
                es_vars.remove(out);
            }
            vals_to_vars(ins) + es_vars
        }
    }
}

pub(crate) fn alpha_convert(map: &HashMap<Var, Var>, exp: &Rc<Exp>) -> Rc<Exp> {
    fn alpha_convert_es(map: &HashMap<Var, Var>, es: &[Rc<Exp>]) -> Box<[Rc<Exp>]> {
        es.iter().map(|x| alpha_convert(map, x)).collect()
    }
    fn alpha_convert_var(map: &HashMap<Var, Var>, var: Var) -> Var {
        map.get(&var).copied().unwrap_or(var)
    }
    fn alpha_convert_vars(map: &HashMap<Var, Var>, vars: &[Var]) -> Box<[Var]> {
        vars.iter().map(|x| alpha_convert_var(map, *x)).collect()
    }
    fn alpha_convert_val(map: &HashMap<Var, Var>, val: &Value) -> Value {
        if let Value::Var(var) = val {
            Value::Var(alpha_convert_var(map, *var))
        } else {
            val.clone()
        }
    }
    fn alpha_convert_vals(map: &HashMap<Var, Var>, vals: &[Value]) -> Box<[Value]> {
        vals.iter().map(|x| alpha_convert_val(map, x)).collect()
    }
    fn alpha_convert_fn(map: &HashMap<Var, Var>, f: &FnDef) -> FnDef {
        let FnDef(var, params, e) = f;
        FnDef(
            alpha_convert_var(map, *var),
            alpha_convert_vars(map, params),
            alpha_convert(map, e),
        )
    }
    fn alpha_convert_fns(map: &HashMap<Var, Var>, fns: &[FnDef]) -> Box<[FnDef]> {
        fns.iter().map(|x| alpha_convert_fn(map, x)).collect()
    }

    Rc::new(match &**exp {
        Exp::Record(vals, var, e) => Exp::Record(
            alpha_convert_vals(map, vals),
            alpha_convert_var(map, *var),
            alpha_convert(map, e),
        ),
        Exp::Select(i, val, var, e) => Exp::Select(
            *i,
            alpha_convert_val(map, val),
            alpha_convert_var(map, *var),
            alpha_convert(map, e),
        ),
        Exp::Offset(i, val, var, e) => Exp::Offset(
            *i,
            alpha_convert_val(map, val),
            alpha_convert_var(map, *var),
            alpha_convert(map, e),
        ),
        Exp::App(f, vals) => Exp::App(alpha_convert_val(map, f), alpha_convert_vals(map, vals)),
        Exp::Fix(fns, e) => Exp::Fix(alpha_convert_fns(map, fns), alpha_convert(map, e)),
        Exp::Switch(val, es) => Exp::Switch(alpha_convert_val(map, val), alpha_convert_es(map, es)),
        Exp::Primop(op, ins, outs, es) => Exp::Primop(
            *op,
            alpha_convert_vals(map, ins),
            alpha_convert_vars(map, outs),
            alpha_convert_es(map, es),
        ),
    })
}

// fn find_escaping_fns(exp: &Exp) -> HashSet<Var> {
//     HashSet::new()
// }

#[cfg(test)]
mod tests {
    use super::*;
    use im::hashset;

    #[test]
    fn test_flv_trivial() {
        let input = Exp::App(Value::Label(Var(0)), Box::new([]));
        let result = free_variables(&input);
        assert_eq!(result, hashset! {});
    }

    #[test]
    fn test_flv_app_f() {
        let input = Exp::App(Value::Var(Var(0)), Box::new([]));
        let result = free_variables(&input);
        assert_eq!(result, hashset! {Var(0)});
    }

    #[test]
    fn test_flv_app_args() {
        let input = Exp::App(Value::Label(Var(0)), Box::new([Value::Var(Var(1))]));
        let result = free_variables(&input);
        assert_eq!(result, hashset! {Var(1)});
    }

    #[test]
    fn test_flv_record_trivial() {
        let input = Exp::Record(
            Box::new([]),
            Var(0),
            Rc::new(Exp::App(Value::Label(Var(0)), Box::new([]))),
        );
        let result = free_variables(&input);
        assert_eq!(result, hashset! {});
    }

    #[test]
    fn test_flv_record_cons() {
        let input = Exp::Record(
            Box::new([Value::Var(Var(1)), Value::Var(Var(2))]),
            Var(0),
            Rc::new(Exp::App(Value::Label(Var(0)), Box::new([]))),
        );
        let result = free_variables(&input);
        assert_eq!(result, hashset! {Var(1), Var(2)});
    }

    #[test]
    fn test_flv_record_exp() {
        let input = Exp::Record(
            Box::new([]),
            Var(0),
            Rc::new(Exp::App(Value::Var(Var(1)), Box::new([]))),
        );
        let result = free_variables(&input);
        assert_eq!(result, hashset! {Var(1)});
    }

    #[test]
    fn test_flv_record_exclude() {
        let input = Exp::Record(
            Box::new([]),
            Var(1),
            Rc::new(Exp::App(Value::Var(Var(1)), Box::new([]))),
        );
        let result = free_variables(&input);
        assert_eq!(result, hashset! {});
    }

    // #[test]
    // fn test_escaping_fns_none() {
    //     let input = Exp::Fix(
    //         Box::new([]),
    //         Rc::new(Exp::App(Value::Label(Var(1)), Box::new([]))),
    //     );
    //     let result = find_escaping_fns(&input);
    //     assert_eq!(result, hashset! {});
    // }

    // #[test]
    // fn test_escaping_fns_record() {
    //     let input = Exp::Fix(
    //         Box::new([FnDef(
    //             Var(0),
    //             Box::new([]),
    //             Rc::new(Exp::App(Value::Label(Var(1)), Box::new([]))),
    //         )]),
    //         Rc::new(Exp::Record(
    //             Box::new([Value::Var(Var(0))]),
    //             Var(2),
    //             Rc::new(Exp::App(Value::Label(Var(3)), Box::new([]))),
    //         )),
    //     );
    //     let result = find_escaping_fns(&input);
    //     assert_eq!(result, hashset! {Var(0)});
    // }
}
