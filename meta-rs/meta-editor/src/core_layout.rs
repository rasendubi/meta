use std::collections::HashMap;

use druid_shell::{HotKey, KeyCode, RawMods};
use im::HashSet;
use itertools::Itertools;

use meta_core::MetaCore;
use meta_store::{Datom, Field};

use crate::key::KeyHandler;
use crate::layout::{
    brackets, concat, datom_reference, datom_value, empty, field, group, line, linebreak, nest,
    parentheses, punctuation, quotes, text, whitespace, with_key_handler, Doc, ReferenceTarget,
    TypeFilter,
};

struct EntityKeys {
    entity: Field,
}

impl EntityKeys {
    fn new(entity: Field) -> Self {
        Self { entity }
    }
}

impl KeyHandler for EntityKeys {
    fn handle_key(&self, key: druid_shell::KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        if HotKey::new(RawMods::Ctrl, KeyCode::Return).matches(key) {
            editor.with_store(|store| {
                store.add_datom(&Datom::eav(self.entity.clone(), "".into(), "".into()));
            });
            return true;
        }

        false
    }
}

struct LanguageKeys {
    language: Field,
}

impl LanguageKeys {
    fn new(language: Field) -> Self {
        Self { language }
    }
}

impl KeyHandler for LanguageKeys {
    fn handle_key(&self, key: druid_shell::KeyEvent, editor: &mut crate::editor::Editor) -> bool {
        if HotKey::new(RawMods::Ctrl, KeyCode::Return).matches(key) {
            editor.with_store(|store| {
                let entity = Field::new_id();
                let language_entity_id = "13".into();
                store.add_datom(&Datom::eav(
                    self.language.clone(),
                    language_entity_id,
                    entity,
                ));
            });
            return true;
        }

        false
    }
}

fn annotate(core: &MetaCore, entity: &Field) -> Doc {
    let identifier = core.identifier(entity).map_or(empty(), datom_value);

    concat(vec![identifier, parentheses(field(entity))])
}

fn reference(core: &MetaCore, atom: &Datom, target: ReferenceTarget) -> Doc {
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

fn core_layout_value(core: &MetaCore, datom: &Datom) -> Doc {
    let attribute_type = core.meta_attribute_type(&datom.attribute).map(|d| &d.value);
    let reference_type = "3".into();
    if attribute_type == Some(&reference_type) {
        reference(core, datom, ReferenceTarget::Value)
    } else {
        quotes(datom_value(datom))
    }
    // TODO: handle NaturalNumber, IntegerNumber
}

fn core_layout_attribute(core: &MetaCore, value_datoms: &HashSet<Datom>) -> Doc {
    concat(
        value_datoms
            .iter()
            .sorted_by_key(|x| &x.id)
            .map(|x| {
                concat(vec![
                    linebreak(),
                    reference(core, x, ReferenceTarget::Attribute),
                    whitespace(" "),
                    punctuation("="),
                    group(nest(2, concat(vec![line(), core_layout_value(core, x)]))),
                ])
            })
            .collect(),
    )
}

pub fn core_layout_entity(core: &MetaCore, entity: &Field) -> Doc {
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
                        .into_iter()
                        .sorted_by(|(a, _), (b, _)| a.cmp(b))
                        .map(|(_attr, datoms)| core_layout_attribute(&core, &datoms))
                        .collect(),
                ),
            ),
            linebreak(),
            punctuation("}"),
        ]),
    )
}

#[allow(dead_code)]
pub fn core_layout_entities(core: &MetaCore) -> Doc {
    let entities = core.store.entities().into_iter().sorted();
    concat(
        entities
            .map(|e| core_layout_entity(core, e))
            .intersperse_with(|| concat(vec![linebreak(), linebreak()]))
            .collect(),
    )
}

pub fn core_layout_datom(core: &MetaCore, datom: &Datom) -> Doc {
    nest(
        2,
        group(concat(vec![
            brackets(field(&datom.id)),
            line(),
            group(concat(vec![
                annotate(core, &datom.entity),
                punctuation("."),
                linebreak(),
                annotate(core, &datom.attribute),
            ])),
            whitespace(" "),
            punctuation("="),
            nest(2, concat(vec![line(), core_layout_value(core, datom)])),
        ])),
    )
}

#[allow(dead_code)]
pub fn core_layout_datoms(core: &MetaCore) -> Doc {
    let mut datoms: Vec<&Datom> = core.store.atoms().values().collect();
    datoms.sort_by_key(|d| &d.id);

    concat(
        datoms
            .iter()
            .map(|d| core_layout_datom(core, d))
            .intersperse_with(linebreak)
            .collect(),
    )
}

pub fn core_layout_language(core: &MetaCore, id: &Field) -> Doc {
    let language_entity_id = "13".into();
    let entities = core
        .store
        .eav2(id, &language_entity_id)
        .cloned()
        .unwrap_or_else(HashSet::new);
    let entities = order(core, entities.iter().collect());

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
                    .map(|e| core_layout_entity(core, &e.value))
                    .intersperse_with(|| concat(vec![linebreak(), linebreak()]))
                    .collect(),
            ),
        ]),
    )
}

pub fn core_layout_languages(core: &MetaCore) -> Doc {
    let language_id = "12".into();
    let languages = core.of_type(&language_id);

    concat(
        languages
            .iter()
            .map(|l| core_layout_language(core, &l.entity))
            .intersperse_with(linebreak)
            .collect(),
    )
}

/// Order atoms in order determines by `after` attribute. If `after` is not specified, order by atom
/// id.
// Believe me or not, it's actually O(n + m*log(m)), where n is the total number of datoms and m is
// the number of atoms without "after" attribute.
fn order<'a>(core: &'a MetaCore, atoms: Vec<&'a Datom>) -> Vec<&'a Datom> {
    let mut no_after = HashSet::new();
    let mut next = HashMap::<&Field, HashSet<&Datom>>::new();
    for x in atoms.iter() {
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
    use meta_store::MetaStore;
    use std::str::FromStr;

    #[test]
    fn test_order_no_after() {
        let store = MetaStore::from_str(
            r#"
              ["10", "0", "1", "2"]
              ["11", "0", "1", "3"]
              ["12", "0", "1", "4"]
            "#,
        )
        .unwrap();
        let core = MetaCore::new(&store);

        let result = order(
            &core,
            store
                .eav2(&"0".into(), &"1".into())
                .map(|x| x.iter().collect())
                .unwrap_or_else(Vec::new),
        );

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
        let store = MetaStore::from_str(
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

        let result = order(
            &core,
            store
                .eav2(&"0".into(), &"1".into())
                .map(|x| x.iter().collect())
                .unwrap_or_else(Vec::new),
        );

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
        let store = MetaStore::from_str(
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

        let result = order(
            &core,
            store
                .eav2(&"0".into(), &"1".into())
                .map(|x| x.iter().collect())
                .unwrap_or_else(Vec::new),
        );

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
