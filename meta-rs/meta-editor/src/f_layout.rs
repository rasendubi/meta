use im::HashMap;
use itertools::Itertools;
use lazy_static::lazy_static;

use meta_core::MetaCore;
use meta_f::ids;
use meta_store::{Field, Store};

use crate::layout::*;

lazy_static! {
    static ref HANDLERS: HashMap<Field, fn(&MetaCore, &Field) -> RDoc> = {
        let mut m = HashMap::<Field, fn(&MetaCore, &Field) -> RDoc>::new();
        m.insert(ids::ENTRY_POINT.clone(), layout_entry_point);
        m.insert(ids::NUMBER_LITERAL.clone(), layout_number_literal);
        m.insert(ids::STRING_LITERAL.clone(), layout_string_literal);
        m.insert(ids::IDENTIFIER.clone(), layout_identifier);
        m.insert(ids::FUNCTION.clone(), layout_function);
        m.insert(ids::PARAMETER.clone(), layout_parameter);
        m.insert(ids::APPLICATION.clone(), layout_application);
        m.insert(ids::BLOCK.clone(), layout_block);
        m.insert(ids::BINDING.clone(), layout_binding);
        m
    };
}

fn f_layout(core: &MetaCore, entity: &Field) -> RDoc {
    let handler = core
        .meta_type(entity)
        .and_then(|type_| HANDLERS.get(&type_.value))
        .copied()
        .unwrap_or(layout_empty);
    handler(core, entity)
}

fn layout_entry_point(core: &MetaCore, entity: &Field) -> RDoc {
    core.store
        .value(entity, &ids::ENTRY_POINT_EXPR)
        .map_or_else(empty, |expr| f_layout(core, &expr.value))
}

fn layout_number_literal(core: &MetaCore, entity: &Field) -> RDoc {
    core.store
        .value(entity, &ids::NUMBER_LITERAL_VALUE)
        .map_or_else(empty, |d| datom_value(d))
}

fn layout_string_literal(core: &MetaCore, entity: &Field) -> RDoc {
    core.store
        .value(entity, &ids::STRING_LITERAL_VALUE)
        .map_or_else(empty, |d| quotes(datom_value(d)))
}

fn layout_identifier(core: &MetaCore, entity: &Field) -> RDoc {
    core.store
        .value(entity, &ids::IDENTIFIER_IDENTIFIER)
        .map_or_else(empty, |d| datom_value(d))
}

fn layout_function(core: &MetaCore, entity: &Field) -> RDoc {
    let params = core.ordered_values(entity, &ids::FUNCTION_PARAMETER);

    group(concat(vec![
        text("fn"), // TODO: keyword
        parentheses(concat(
            params
                .iter()
                .map(|d| f_layout(core, &d.value))
                .intersperse_with(|| concat(vec![punctuation(","), whitespace(" ")])),
        )),
        whitespace(" "),
        punctuation("->"),
        line(),
        core.store
            .value(entity, &ids::FUNCTION_BODY)
            .map_or_else(empty, |d| f_layout(core, &d.value)),
    ]))
}

fn layout_application(core: &MetaCore, entity: &Field) -> RDoc {
    let args = core.ordered_values(entity, &ids::APPLICATION_ARGUMENT);

    group(concat(vec![
        parentheses(
            core.store
                .value(entity, &ids::APPLICATION_FN)
                .map_or_else(empty, |d| f_layout(core, &d.value)),
        ),
        parentheses(concat(
            args.iter()
                .map(|d| f_layout(core, &d.value))
                .intersperse_with(|| concat(vec![punctuation(","), whitespace(" ")])),
        )),
    ]))
}

fn layout_block(core: &MetaCore, entity: &Field) -> RDoc {
    let stmts = core.ordered_values(entity, &ids::BLOCK_STATEMENT);

    group(braces(concat(vec![
        nest(
            2,
            concat(
                stmts
                    .iter()
                    .map(|stmt| concat(vec![line(), f_layout(core, &stmt.value)]))
                    .intersperse_with(|| punctuation(";")),
            ),
        ),
        line(),
    ])))
}

fn layout_binding(core: &MetaCore, entity: &Field) -> RDoc {
    group(concat(vec![
        core.store
            .value(entity, &ids::BINDING_IDENTIFIER)
            .map_or_else(empty, |d| f_layout(core, &d.value)),
        whitespace(" "),
        punctuation("="),
        line(),
        core.store
            .value(entity, &ids::BINDING_VALUE)
            .map_or_else(empty, |d| f_layout(core, &d.value)),
    ]))
}

fn layout_parameter(core: &MetaCore, entity: &Field) -> RDoc {
    core.store
        .value(entity, &ids::PARAMETER_IDENTIFIER)
        .map_or_else(empty, |d| f_layout(core, &d.value))
}

fn layout_empty(_core: &MetaCore, _entity: &Field) -> RDoc {
    empty()
}

pub fn f_layout_entries(store: &Store) -> RDoc {
    let core = MetaCore::new(store);

    let entries = core.of_type(&ids::ENTRY_POINT);

    concat(
        entries
            .iter()
            .sorted()
            .map(|e| f_layout(&core, &e.entity))
            .intersperse_with(linebreak),
    )
}
