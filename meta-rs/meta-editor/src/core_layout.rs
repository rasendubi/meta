use std::collections::HashMap;

use druid_shell::{HotKey, KeyCode, KeyEvent, SysMods};
use im::HashSet;
use itertools::Itertools;

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
        ReferenceTarget::Attribute => {
            let attribute_id = "7".into();
            TypeFilter::from_type(attribute_id)
        }
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
    let reference_type = "3".into();
    if attribute_type == Some(&reference_type) {
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
                .map(|e| with_id(vec![e.clone()], core_layout_entity(&core, e)))
                .intersperse_with(|| concat(vec![linebreak(), linebreak()])),
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
                .map(|d| with_id(vec![d.id.clone()], core_layout_datom(&core, d)))
                .intersperse_with(linebreak),
        ),
    )
}

pub fn core_layout_language(core: &MetaCore, id: &Field) -> RDoc {
    let language_entity_id = "13".into();
    let entities = core
        .store
        .eav2(id, &language_entity_id)
        .map_or_else(Vec::new, |e| order(core, e));

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
                        with_id(
                            vec![id.clone(), e.value.clone()],
                            core_layout_entity(core, &e.value),
                        )
                        .with_key(e.value.to_string())
                    })
                    .intersperse_with(|| concat(vec![linebreak(), linebreak()])),
            ),
        ]),
    )
}

pub fn core_layout_languages(store: &Store) -> RDoc {
    let core = MetaCore::new(store);

    let language_id = "12".into();
    let languages = core.of_type(&language_id);

    concat(
        languages
            .iter()
            .sorted()
            .map(|l| core_layout_language(&core, &l.entity))
            .intersperse_with(|| concat(vec![linebreak(), linebreak()])),
    )
}

/// Order atoms in order determines by `after` attribute. If `after` is not specified, order by atom
/// id.
// Believe me or not, it's actually O(n + m*log(m)), where n is the total number of datoms and m is
// the number of atoms without "after" attribute.
fn order<'a, I: IntoIterator<Item = &'a Datom>>(core: &'a MetaCore, atoms: I) -> Vec<&'a Datom> {
    let mut no_after = HashSet::new();
    let mut next = HashMap::<&Field, HashSet<&Datom>>::new();
    for x in atoms.into_iter() {
        if let Some(a) = core.after(x) {
            next.entry(a).or_insert_with(HashSet::new).insert(x);
        } else {
            no_after.insert(x);
        }
    }

    // it would be much easier if Rust allowed recursive closures
    fn traverse_atom<'a>(
        x: &'a Datom,
        result: &'_ mut Vec<&'a Datom>,
        next: &HashMap<&'a Field, HashSet<&'a Datom>>,
    ) {
        result.push(x);
        if let Some(next_atoms) = next.get(&x.id) {
            for a in next_atoms.iter() {
                traverse_atom(a, result, next);
            }
        }
    }

    let mut result = Vec::new();
    for a in no_after.iter().sorted_by_key(|x| &x.id) {
        traverse_atom(a, &mut result, &next);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use meta_store::Store;
    use std::str::FromStr;

    #[test]
    fn test_order_no_after() {
        let store = Store::from_str(
            r#"
              ["10", "0", "1", "2"]
              ["11", "0", "1", "3"]
              ["12", "0", "1", "4"]
            "#,
        )
        .unwrap();
        let core = MetaCore::new(&store);

        let result = store
            .eav2(&"0".into(), &"1".into())
            .map_or_else(Vec::new, |x| order(&core, x));

        assert_eq!(
            vec![
                &("10", "0", "1", "2").into(),
                &("11", "0", "1", "3").into(),
                &("12", "0", "1", "4").into(),
            ] as Vec<&Datom>,
            result
        );
    }

    #[test]
    fn test_order_with_after() {
        let store = Store::from_str(
            r#"
              ["10", "0", "1", "2"]
              ["11", "0", "1", "3"]
              ["12", "0", "1", "4"]
              ["13", "12", "16", "10"]
              ["14", "11", "16", "12"]
            "#,
        )
        .unwrap();
        let core = MetaCore::new(&store);

        let result = store
            .eav2(&"0".into(), &"1".into())
            .map_or_else(Vec::new, |x| order(&core, x));

        assert_eq!(
            vec![
                &("10", "0", "1", "2").into(),
                &("12", "0", "1", "4").into(),
                &("11", "0", "1", "3").into(),
            ] as Vec<&Datom>,
            result
        );
    }

    #[test]
    #[ignore] // TODO: order silently drops all loops now (a after b, b after a)
    fn test_order_with_after_loop() {
        let store = Store::from_str(
            r#"
              ["10", "0", "1", "2"]
              ["11", "0", "1", "3"]
              ["12", "0", "1", "4"]
              ["13", "12", "16", "10"]
              ["14", "11", "16", "12"]
              ["15", "10", "16", "11"]
            "#,
        )
        .unwrap();
        let core = MetaCore::new(&store);

        let result = store
            .eav2(&"0".into(), &"1".into())
            .map_or_else(Vec::new, |x| order(&core, x));

        // if loop is detected, prefer starting from the lowest id
        assert_eq!(
            vec![
                &("10", "0", "1", "2").into(),
                &("12", "0", "1", "4").into(),
                &("11", "0", "1", "3").into(),
            ] as Vec<&Datom>,
            result
        );
    }
}
