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
    static ref HANDLERS: HashMap<Field, fn(&MetaCore, &Datom) -> RDoc> = {
        let mut m = HashMap::<Field, fn(&MetaCore, &Datom) -> RDoc>::new();
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
        m.insert(ids::TYPEDEF.clone(), layout_typedef);
        m.insert(ids::CONSTRUCTOR.clone(), layout_constructor);
        m.insert(ids::ACCESS.clone(), layout_access);
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
struct BlockKeys(Field);
impl KeyHandler for BlockKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let block = &self.0;
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let id = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    block.clone(),
                    ids::BLOCK_STATEMENT.clone(),
                    id.clone(),
                ));
            });

            editor.goto_cell_id(&[id]);

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct TypeDefKeys(Field);
impl KeyHandler for TypeDefKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let typedef = &self.0;
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let id = Field::new_id();
            let id_id = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    typedef.clone(),
                    ids::TYPEDEF_CONSTRUCTOR.clone(),
                    id.clone(),
                ));

                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::CONSTRUCTOR.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::CONSTRUCTOR_IDENTIFIER.clone(),
                    id_id.clone(),
                ));

                store.add_datom(&Datom::eav(
                    id_id.clone(),
                    core::A_TYPE.clone(),
                    ids::IDENTIFIER.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id_id.clone(),
                    ids::IDENTIFIER_IDENTIFIER.clone(),
                    "".into(),
                ));
            });

            editor.goto_cell_id(&[id_id]);

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct FunctionParamsKeys(Field);
impl KeyHandler for FunctionParamsKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let f = &self.0;
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let param = Field::new_id();
            let identifier = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    f.clone(),
                    ids::FUNCTION_PARAMETER.clone(),
                    param.clone(),
                ));
                store.add_datom(&Datom::eav(
                    param.clone(),
                    core::A_TYPE.clone(),
                    ids::PARAMETER.clone(),
                ));
                store.add_datom(&Datom::eav(
                    param.clone(),
                    ids::PARAMETER_IDENTIFIER.clone(),
                    identifier.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    core::A_TYPE.clone(),
                    ids::IDENTIFIER.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    ids::IDENTIFIER_IDENTIFIER.clone(),
                    "".into(),
                ));
            });

            editor.goto_cell_id(&[identifier]);

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct ConstructorParamsKeys(Field);
impl KeyHandler for ConstructorParamsKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let f = &self.0;
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let param = Field::new_id();
            let identifier = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    f.clone(),
                    ids::CONSTRUCTOR_PARAMETER.clone(),
                    param.clone(),
                ));
                store.add_datom(&Datom::eav(
                    param.clone(),
                    core::A_TYPE.clone(),
                    ids::PARAMETER.clone(),
                ));
                store.add_datom(&Datom::eav(
                    param.clone(),
                    ids::PARAMETER_IDENTIFIER.clone(),
                    identifier.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    core::A_TYPE.clone(),
                    ids::IDENTIFIER.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    ids::IDENTIFIER_IDENTIFIER.clone(),
                    "".into(),
                ));
            });

            editor.goto_cell_id(&[identifier]);

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct ApplicationArgsKeys(Field);
impl KeyHandler for ApplicationArgsKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let f = &self.0;
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let arg = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    f.clone(),
                    ids::APPLICATION_ARGUMENT.clone(),
                    arg.clone(),
                ));
            });

            editor.goto_cell_id(&[arg]);

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct EntityKeys(Datom);
impl KeyHandler for EntityKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let core = MetaCore::new(editor.store());
        let id = &self.0.value;
        let my_type = core.meta_type(id).map(|d| &d.value);

        if HotKey::new(SysMods::Cmd, KeyCode::KeyD).matches(key) {
            let new_id = Field::new_id();

            if &self.0.attribute == &ids::FUNCTION_PARAMETER as &Field
                || &self.0.attribute == &ids::BLOCK_STATEMENT as &Field
                || &self.0.attribute == &ids::APPLICATION_ARGUMENT as &Field
                || my_type == Some(&ids::RUN_TEST)
            {
                // delete completely
                editor.with_store(|store| {
                    store.remove_datom(&self.0);
                });
            } else {
                // just replace with a hole
                let my_type = my_type.cloned();
                editor.with_store(|store| {
                    store.remove_datom(&self.0);
                    let mut new_datom = self.0.clone();
                    new_datom.value = new_id.clone();
                    store.add_datom(&new_datom);

                    if my_type.as_ref() == Some(&ids::IDENTIFIER) {
                        // Identifier can only occur at specific context where Identifier is always
                        // expected and no other type makes sense.
                        store.add_datom(&Datom::eav(
                            new_id.clone(),
                            core::A_TYPE.clone(),
                            my_type.unwrap(),
                        ));
                    }
                });

                editor.goto_cell_id(&[new_id]);
            }

            return true;
        }

        false
    }
}

#[derive(Debug)]
struct HoleKeys(Datom);
impl KeyHandler for HoleKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        let core = MetaCore::new(editor.store());
        let parent = &self.0.entity;
        let parent_type = core.meta_type(parent).map(|d| &d.value);

        let id = &self.0.value;
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

        if HotKey::new(None, "t").matches(key) {
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::TYPEDEF.clone(),
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

        if HotKey::new(None, ".").matches(key) {
            let object = Field::new_id();
            let identifier = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::ACCESS.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::ACCESS_OBJECT.clone(),
                    object.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::ACCESS_FIELD.clone(),
                    identifier.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    core::A_TYPE.clone(),
                    ids::IDENTIFIER_REFERENCE.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    ids::IDENTIFIER_REFERENCE_IDENTIFIER.clone(),
                    "".into(),
                ));
            });

            editor.goto_cell_id(&[object]);

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

        if parent_type == Some(&ids::BLOCK as &Field)
            && (HotKey::new(None, "=").matches(key)
                || HotKey::new(SysMods::Shift, "=").matches(key))
        {
            let identifier = Field::new_id();

            editor.with_store(|store| {
                store.add_datom(&Datom::eav(
                    id.clone(),
                    core::A_TYPE.clone(),
                    ids::BINDING.clone(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::BINDING_IDENTIFIER.clone(),
                    identifier.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    core::A_TYPE.clone(),
                    ids::IDENTIFIER.clone(),
                ));
                store.add_datom(&Datom::eav(
                    identifier.clone(),
                    ids::IDENTIFIER_IDENTIFIER.clone(),
                    "".into(),
                ));
                store.add_datom(&Datom::eav(
                    id.clone(),
                    ids::BINDING_VALUE.clone(),
                    Field::new_id(),
                ));
            });

            editor.goto_cell_id(&[identifier]);

            return true;
        }

        false
    }
}

fn f_layout(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    let handler = core
        .meta_type(entity)
        .and_then(|type_| HANDLERS.get(&type_.value))
        .copied()
        .unwrap_or(layout_hole);
    with_key_handler(
        Box::new(EntityKeys(datom.clone())),
        with_id(vec![entity.clone()], handler(core, datom)),
    )
}

fn layout_number_literal(core: &MetaCore, datom: &Datom) -> RDoc {
    core.store
        .value(&datom.value, &ids::NUMBER_LITERAL_VALUE)
        .map_or_else(empty, |d| datom_value(d))
}

fn layout_string_literal(core: &MetaCore, datom: &Datom) -> RDoc {
    core.store
        .value(&datom.value, &ids::STRING_LITERAL_VALUE)
        .map_or_else(empty, |d| quotes(datom_value(d)))
}

fn layout_identifier(core: &MetaCore, datom: &Datom) -> RDoc {
    core.store
        .value(&datom.value, &ids::IDENTIFIER_IDENTIFIER)
        .map_or_else(empty, |d| datom_value(d))
}

fn layout_identifier_reference(core: &MetaCore, datom: &Datom) -> RDoc {
    core.store
        .value(&datom.value, &ids::IDENTIFIER_REFERENCE_IDENTIFIER)
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

fn layout_function(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    let params = core.ordered_values(entity, &ids::FUNCTION_PARAMETER);

    group(concat(vec![
        text("fn"), // TODO: keyword
        with_key_handler(
            Box::new(FunctionParamsKeys(entity.clone())),
            parentheses(concat(
                params
                    .iter()
                    .map(|d| f_layout(core, d))
                    .intersperse_with(|| concat(vec![punctuation(","), whitespace(" ")])),
            )),
        ),
        whitespace(" "),
        punctuation("->"),
        line(),
        core.store
            .value(entity, &ids::FUNCTION_BODY)
            .map_or_else(empty, |d| f_layout(core, d)),
    ]))
}

fn layout_application(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    let args = core.ordered_values(entity, &ids::APPLICATION_ARGUMENT);

    group(concat(vec![
        parentheses(
            core.store
                .value(entity, &ids::APPLICATION_FN)
                .map_or_else(empty, |d| f_layout(core, d)),
        ),
        with_key_handler(
            Box::new(ApplicationArgsKeys(entity.clone())),
            parentheses(concat(
                args.iter()
                    .map(|d| f_layout(core, d))
                    .intersperse_with(|| concat(vec![punctuation(","), whitespace(" ")])),
            )),
        ),
    ]))
}

fn layout_block(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    let stmts = core.ordered_values(entity, &ids::BLOCK_STATEMENT);

    with_key_handler(
        Box::new(BlockKeys(entity.clone())),
        group(braces(concat(vec![
            nest(
                2,
                concat(
                    stmts
                        .iter()
                        .map(|stmt| {
                            concat(vec![line(), f_layout(core, stmt)]).with_key(stmt.id.to_string())
                        })
                        .intersperse_with(|| punctuation(";")),
                ),
            ),
            line(),
        ]))),
    )
}

fn layout_binding(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    group(concat(vec![
        core.store
            .value(entity, &ids::BINDING_IDENTIFIER)
            .map_or_else(empty, |d| f_layout(core, d)),
        whitespace(" "),
        punctuation("="),
        line(),
        core.store
            .value(entity, &ids::BINDING_VALUE)
            .map_or_else(empty, |d| f_layout(core, d)),
    ]))
}

fn layout_parameter(core: &MetaCore, datom: &Datom) -> RDoc {
    core.store
        .value(&datom.value, &ids::PARAMETER_IDENTIFIER)
        .map_or_else(empty, |d| f_layout(core, d))
}

fn layout_typedef(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    let constructors = core.ordered_values(entity, &ids::TYPEDEF_CONSTRUCTOR);

    with_key_handler(
        Box::new(TypeDefKeys(entity.clone())),
        concat(vec![
            text("type"),
            whitespace(" "),
            group(braces(concat(vec![
                line(),
                nest(
                    2,
                    concat(
                        constructors
                            .iter()
                            .map(|c| f_layout(core, c).with_key(c.id.to_string()))
                            .intersperse_with(|| {
                                concat(vec![line(), punctuation("|"), whitespace(" ")])
                            }),
                    ),
                ),
                line(),
            ]))),
        ]),
    )
}

fn layout_constructor(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    let id = core
        .store
        .value(entity, &ids::CONSTRUCTOR_IDENTIFIER)
        .map_or_else(empty, |i| f_layout(core, i));

    let params = core.ordered_values(entity, &ids::CONSTRUCTOR_PARAMETER);

    concat(vec![
        id,
        with_key_handler(
            Box::new(ConstructorParamsKeys(entity.clone())),
            parentheses(nest(
                2,
                concat(
                    params
                        .iter()
                        .map(|p| f_layout(core, p))
                        .intersperse_with(|| concat(vec![punctuation(","), whitespace(" ")])),
                ),
            )),
        ),
    ])
}

fn layout_access(core: &MetaCore, datom: &Datom) -> RDoc {
    let entity = &datom.value;
    let object = core
        .store
        .value(entity, &ids::ACCESS_OBJECT)
        .map_or_else(empty, |d| f_layout(core, d));
    let identifier = core
        .store
        .value(entity, &ids::ACCESS_FIELD)
        .map_or_else(empty, |d| f_layout(core, d));

    concat(vec![object, punctuation("."), identifier])
}

fn layout_hole(_core: &MetaCore, datom: &Datom) -> RDoc {
    with_key_handler(Box::new(HoleKeys(datom.clone())), text("_"))
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
                            .map_or_else(empty, |expr| f_layout(core, expr)),
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
                    with_id(vec![e.entity.clone()], layout_run_test(&core, &e.entity))
                        .with_key(e.entity.to_string())
                })
                .intersperse_with(linebreak),
        ),
    )
}
