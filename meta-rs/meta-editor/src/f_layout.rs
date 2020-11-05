use druid_shell::{HotKey, KeyCode, KeyEvent, SysMods};
use im::HashMap;
use itertools::Itertools;
use lazy_static::lazy_static;

use meta_core::ids as core;
use meta_core::MetaCore;
use meta_f::ids;
use meta_store::{Datom, Field, Store};

use crate::key::KeyHandler;
use crate::layout::*;

lazy_static! {
    static ref HANDLERS: HashMap<Field, fn(&MetaCore, &Field) -> RDoc> = {
        let mut m = HashMap::<Field, fn(&MetaCore, &Field) -> RDoc>::new();
        m.insert(ids::RUN_TEST.clone(), layout_run_test);
        m.insert(ids::NUMBER_LITERAL.clone(), layout_number_literal);
        m.insert(ids::STRING_LITERAL.clone(), layout_string_literal);
        m.insert(ids::IDENTIFIER.clone(), layout_identifier);
        m.insert(
            ids::IDENTIFIER_REFERENCE.clone(),
            layout_identifier_reference,
        );
        m.insert(ids::FUNCTION.clone(), layout_function);
        m.insert(ids::PARAMETER.clone(), layout_parameter);
        m.insert(ids::APPLICATION.clone(), layout_application);
        m.insert(ids::BLOCK.clone(), layout_block);
        m.insert(ids::BINDING.clone(), layout_binding);
        m
    };
}

#[derive(Debug)]
struct FKeys;
impl KeyHandler for FKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let id = editor.with_store(|store| {
                let test = Field::new_id();

                store.add_datom(&Datom::eav(
                    test.clone(),
                    core::A_TYPE.clone(),
                    ids::RUN_TEST.clone(),
                ));
                store.add_datom(&Datom::eav(
                    test.clone(),
                    core::A_IDENTIFIER.clone(),
                    "".into(),
                ));
                store.add_datom(&Datom::eav(
                    test.clone(),
                    ids::RUN_TEST_EXPECTED_RESULT.clone(),
                    "".into(),
                ));

                let expr = Field::new_id();
                store.add_datom(&Datom::eav(test.clone(), ids::RUN_TEST_EXPR.clone(), expr));

                test
            });

            editor.goto_cell_id(&[id]);

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct RunTestKeys(Field);
impl KeyHandler for RunTestKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        if HotKey::new(None, KeyCode::F3).matches(key) {
            let test = &self.0;
            let result = meta_f::interpret(editor.store(), test);

            editor.with_store(|store| {
                let id =
                    if let Some(datom) = store.value(test, &ids::RUN_TEST_ACTUAL_RESULT).cloned() {
                        store.remove_datom(&datom);
                        datom.id
                    } else {
                        Field::new_id()
                    };

                store.add_datom(&Datom::new(
                    id,
                    test.clone(),
                    ids::RUN_TEST_ACTUAL_RESULT.clone(),
                    Field::from(format!("{:?}", result)),
                ));
            });

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct HoleKeys(Field);
impl KeyHandler for HoleKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let id = &self.0;
        if HotKey::new(None, KeyCode::Key0).matches(key)
            || HotKey::new(None, KeyCode::Key1).matches(key)
            || HotKey::new(None, KeyCode::Key2).matches(key)
            || HotKey::new(None, KeyCode::Key3).matches(key)
            || HotKey::new(None, KeyCode::Key4).matches(key)
            || HotKey::new(None, KeyCode::Key5).matches(key)
            || HotKey::new(None, KeyCode::Key6).matches(key)
            || HotKey::new(None, KeyCode::Key7).matches(key)
            || HotKey::new(None, KeyCode::Key8).matches(key)
            || HotKey::new(None, KeyCode::Key9).matches(key)
        {
            let digit = key.text().expect("digit key must yield text");

            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::NUMBER_LITERAL.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::NUMBER_LITERAL_VALUE.clone(),
                    Field::from(digit),
                ));
            });

            return true;
        }

        if HotKey::new(None, KeyCode::Quote).matches(key)
            || HotKey::new(SysMods::Shift, KeyCode::Quote).matches(key)
        {
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::STRING_LITERAL.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::STRING_LITERAL_VALUE.clone(),
                    "".into(),
                ));
            });

            return true;
        }

        // TODO: druid does not have an option to ignore shift modifier
        if HotKey::new(None, "{").matches(key) || HotKey::new(SysMods::Shift, "{").matches(key) {
            let expr = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::BLOCK.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::BLOCK_STATEMENT.clone(),
                    expr.clone(),
                ));
            });

            editor.goto_cell_id(&[expr]);

            return true;
        }

        if HotKey::new(None, "f").matches(key) {
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::FUNCTION.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::FUNCTION_BODY.clone(),
                    Field::new_id(),
                ));
            });

            return true;
        }

        if HotKey::new(None, "(").matches(key) || HotKey::new(SysMods::Shift, "(").matches(key) {
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::APPLICATION.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::APPLICATION_FN.clone(),
                    Field::new_id(),
                ));
            });

            return true;
        }

        if HotKey::new(None, "&").matches(key) || HotKey::new(SysMods::Shift, "&").matches(key) {
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::IDENTIFIER_REFERENCE.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::IDENTIFIER_REFERENCE_IDENTIFIER.clone(),
                    "".into(),
                ));
            });

            return true;
        }

        false
    }
}

fn f_layout(core: &MetaCore, entity: &Field) -> RDoc {
    let handler = core
        .meta_type(entity)
        .and_then(|type_| HANDLERS.get(&type_.value))
        .copied()
        .unwrap_or(layout_hole);
    with_id(vec![entity.clone()], handler(core, entity))
}

fn layout_run_test(core: &MetaCore, entity: &Field) -> RDoc {
    with_key_handler(
        Box::new(RunTestKeys(entity.clone())),
        concat(vec![
            core.identifier(entity).map_or_else(empty, datom_value),
            whitespace(" "),
            group(braces(concat(vec![
                nest(
                    2,
                    concat(vec![
                        line(),
                        text("expression"),
                        whitespace(" "),
                        punctuation("="),
                        whitespace(" "),
                        core.store
                            .value(entity, &ids::RUN_TEST_EXPR)
                            .map_or_else(empty, |expr| f_layout(core, &expr.value)),
                        punctuation(";"),
                        line(),
                        text("expected result"),
                        whitespace(" "),
                        punctuation("="),
                        whitespace(" "),
                        brackets(
                            core.store
                                .value(entity, &ids::RUN_TEST_EXPECTED_RESULT)
                                .map_or_else(empty, datom_value),
                        ),
                        punctuation(";"),
                        core.store
                            .value(entity, &ids::RUN_TEST_ACTUAL_RESULT)
                            .map_or_else(empty, |expr| {
                                concat(vec![
                                    line(),
                                    text("actual result"),
                                    whitespace(" "),
                                    punctuation("="),
                                    whitespace(" "),
                                    brackets(datom_value(expr)),
                                    punctuation(";"),
                                ])
                            }),
                    ]),
                ),
                line(),
            ]))),
            linebreak(),
        ]),
    )
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

fn layout_identifier_reference(core: &MetaCore, entity: &Field) -> RDoc {
    core.store
        .value(entity, &ids::IDENTIFIER_REFERENCE_IDENTIFIER)
        .map_or_else(empty, |d| {
            let empty = Field::from("");
            let value = core
                .store
                .value(&d.value, &ids::IDENTIFIER_IDENTIFIER)
                .map_or(&empty, |d| &d.value);
            datom_reference(
                d,
                ReferenceTarget::Value,
                TypeFilter::from_type(ids::IDENTIFIER.clone()),
                value,
            )
        })
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
        core.store
            .value(entity, &ids::APPLICATION_FN)
            .map_or_else(empty, |d| f_layout(core, &d.value)),
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

fn layout_hole(_core: &MetaCore, entity: &Field) -> RDoc {
    with_key_handler(Box::new(HoleKeys(entity.clone())), text("_"))
}

pub fn f_layout_entries(store: &Store) -> RDoc {
    let core = MetaCore::new(store);

    let entries = core.of_type(&ids::RUN_TEST);

    with_key_handler(
        Box::new(FKeys),
        concat(
            entries
                .iter()
                .sorted()
                .map(|e| {
                    with_id(vec![e.entity.clone()], f_layout(&core, &e.entity))
                        .with_key(e.entity.to_string())
                })
                .intersperse_with(linebreak),
        ),
    )
}
