use druid_shell::{HotKey, KeyCode, KeyEvent, SysMods};
use im::HashSet;
use itertools::Itertools;

use meta_core::ids;
use meta_core::MetaCore;
use meta_store::{Datom, Field, Store};

use crate::editor::Editor;
use crate::key::KeyHandler;
use crate::layout::*;

#[derive(Debug)]
struct EntityKeys {
    entity: Field,
}
impl EntityKeys {
    fn new(entity: Field) -> Self {
        Self { entity }
    }
}
impl KeyHandler for EntityKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool {
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let id = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::new(
                    id.clone(),
                    self.entity.clone(),
                    "".into(),
                    "".into(),
                ));
            });

            goto_cell_id(editor, &[self.entity.clone(), id]);
            return true;
        }

        if HotKey::new(SysMods::Cmd, KeyCode::KeyD).matches(key) {
            editor.with_store(|store| {
                let datoms = store.eav1(&self.entity).map_or_else(HashSet::new, |attrs| {
                    HashSet::unions(attrs.values().cloned())
                });
                for datom in datoms {
                    store.remove_datom(&datom);
                }
            });
            return true;
        }

        false
    }
}

#[derive(Debug)]
struct LanguageKeys {
    language: Field,
}
impl LanguageKeys {
    fn new(language: Field) -> Self {
        Self { language }
    }
}
impl KeyHandler for LanguageKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool {
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let entity = Field::new_id();

            editor.with_store(|store| {
                let language_entity_id = "13".into();
                store.add_datom(&Datom::eav(
                    self.language.clone(),
                    language_entity_id,
                    entity.clone(),
                ));
            });

            goto_cell_id(editor, &[self.language.clone(), entity]);
            return true;
        }

        false
    }
}

#[derive(Debug)]
struct EntitiesKeys;
impl KeyHandler for EntitiesKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool {
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let entity = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(entity.clone(), "".into(), "".into()))
            });

            goto_cell_id(editor, &[entity]);
            return true;
        }

        false
    }
}

#[derive(Debug)]
struct DatomsKeys;
impl KeyHandler for DatomsKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool {
        if HotKey::new(SysMods::Cmd, KeyCode::Return).matches(key) {
            let id = Field::new_id();
            editor.with_store(|store| {
                store.add_datom(&Datom::new(id.clone(), "".into(), "".into(), "".into()))
            });

            goto_cell_id(editor, &[id]);
            return true;
        }

        false
    }
}

#[derive(Debug)]
struct DatomKeys(Field);
impl KeyHandler for DatomKeys {
    fn handle_key(&self, key: KeyEvent, editor: &mut Editor) -> bool {
        if HotKey::new(SysMods::Cmd, KeyCode::KeyD).matches(key) {
            editor.with_store(|store| {
                if let Some(datom) = store.atoms().get(&self.0).cloned() {
                    store.remove_datom(&datom);
                }
            });
            return true;
        }

        false
    }
}

fn annotate(core: &MetaCore, entity: &Field) -> RDoc {
    let identifier = core.identifier(entity).map_or(empty(), datom_value);

    concat(vec![identifier, parentheses(field(entity))])
}

fn reference(core: &MetaCore, atom: &Datom, target: ReferenceTarget) -> RDoc {
    let type_filter = match target {
        ReferenceTarget::Attribute => TypeFilter::from_type(ids::T_ATTRIBUTE.clone()),
        ReferenceTarget::Value => TypeFilter::new(
            core.meta_attribute_reference_type(&atom.attribute)
                .map(|hs| hs.iter().map(|datom| datom.value.clone()).collect()),
        ),
        ReferenceTarget::Id | ReferenceTarget::Entity => TypeFilter::unfiltered(),
    };

    let entity = target.get_field(atom);
    match core.identifier(entity) {
        Some(identifier) => datom_reference(atom, target, type_filter, &identifier.value),
        None => {
            let mut s = String::new();
            s += "(";
            s += entity.as_ref();
            s += ")";

            datom_reference(atom, target, type_filter, &s.into())
        }
    }
}

fn core_layout_value(core: &MetaCore, datom: &Datom) -> RDoc {
    let attribute_type = core.meta_attribute_type(&datom.attribute).map(|d| &d.value);
    if attribute_type == Some(&ids::V_REFERENCE) {
        reference(core, datom, ReferenceTarget::Value)
    } else {
        quotes(datom_value(datom))
    }
    // TODO: handle NaturalNumber, IntegerNumber
}

fn core_layout_attribute(core: &MetaCore, datom: &Datom) -> RDoc {
    with_key_handler(
        Box::new(DatomKeys(datom.id.clone())),
        concat(vec![
            linebreak(),
            reference(core, datom, ReferenceTarget::Attribute),
            whitespace(" "),
            punctuation("="),
            group(nest(
                2,
                concat(vec![line(), core_layout_value(core, datom)]),
            )),
        ]),
    )
}

pub fn core_layout_entity(core: &MetaCore, entity: &Field) -> RDoc {
    let attributes = core
        .store
        .eav1(entity)
        .cloned()
        .unwrap_or_else(im::HashMap::new);

    let type_ = core.meta_type(entity);

    with_key_handler(
        Box::new(EntityKeys::new(entity.clone())),
        concat(vec![
            annotate(core, entity),
            whitespace(" "),
            type_.map_or_else(empty, |x| {
                concat(vec![
                    punctuation(":"),
                    whitespace(" "),
                    reference(core, x, ReferenceTarget::Value),
                    whitespace(" "),
                ])
            }),
            punctuation("{"),
            nest(
                2,
                concat(
                    attributes
                        .values()
                        .fold(HashSet::new(), |acc, x| acc.union(x.clone()))
                        .into_iter()
                        .sorted_by(|a, b| (&a.attribute, &a.id).cmp(&(&b.attribute, &b.id)))
                        .map(|datom| {
                            with_id(
                                vec![datom.entity.clone(), datom.id.clone()],
                                core_layout_attribute(&core, &datom),
                            )
                            .with_key(datom.id.to_string())
                        }),
                ),
            ),
            linebreak(),
            punctuation("}"),
        ]),
    )
}

pub fn core_layout_entities(store: &Store) -> RDoc {
    let core = MetaCore::new(store);
    let entities = core.store.entities().into_iter().sorted();
    with_key_handler(
        Box::new(EntitiesKeys),
        concat(
            entities
                .map(|e| {
                    concat(vec![
                        with_id(vec![e.clone()], core_layout_entity(&core, e)),
                        linebreak(),
                    ])
                })
                .intersperse_with(linebreak),
        ),
    )
}

pub fn core_layout_datom(core: &MetaCore, datom: &Datom) -> RDoc {
    with_key_handler(
        Box::new(DatomKeys(datom.id.clone())),
        nest(
            2,
            group(concat(vec![
                brackets(field(&datom.id)),
                line(),
                group(concat(vec![
                    reference(core, datom, ReferenceTarget::Entity),
                    punctuation("."),
                    linebreak(),
                    reference(core, datom, ReferenceTarget::Attribute),
                ])),
                whitespace(" "),
                punctuation("="),
                nest(2, concat(vec![line(), core_layout_value(core, datom)])),
            ])),
        ),
    )
}

pub fn core_layout_datoms(store: &Store) -> RDoc {
    let core = MetaCore::new(store);
    with_key_handler(
        Box::new(DatomsKeys),
        concat(
            core.store
                .atoms()
                .values()
                .sorted_by_key(|d| &d.id)
                .map(|d| {
                    concat(vec![
                        with_id(vec![d.id.clone()], core_layout_datom(&core, d)),
                        linebreak(),
                    ])
                }),
        ),
    )
}

pub fn core_layout_language(core: &MetaCore, id: &Field) -> RDoc {
    let entities = core.ordered_values(id, &ids::A_LANGUAGE_ENTITY);

    with_key_handler(
        Box::new(LanguageKeys::new(id.clone())),
        concat(vec![
            text("language"),
            whitespace(" "),
            annotate(core, id),
            linebreak(),
            linebreak(),
            concat(
                entities
                    .iter()
                    .map(|e| {
                        concat(vec![
                            with_id(
                                vec![id.clone(), e.value.clone()],
                                core_layout_entity(core, &e.value),
                            )
                            .with_key(e.value.to_string()),
                            linebreak(),
                        ])
                    })
                    .intersperse_with(linebreak),
            ),
        ]),
    )
}

pub fn core_layout_languages(store: &Store) -> RDoc {
    let core = MetaCore::new(store);

    let languages = core.of_type(&ids::T_LANGUAGE);

    concat(
        languages
            .iter()
            .sorted()
            .map(|l| core_layout_language(&core, &l.entity))
            .intersperse_with(linebreak),
    )
}
