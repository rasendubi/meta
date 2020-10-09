use im::HashSet;

use itertools::Itertools;
use maplit::hashset;

use meta_core::MetaCore;
use meta_pretty::RichDoc;
use meta_store::{Datom, Field};

use crate::layout::{datom_value, field, line, punctuation, text, whitespace, EditorCellPayload};
use std::collections::HashMap;

type Doc = RichDoc<EditorCellPayload>;

fn surround(left: Doc, right: Doc, doc: Doc) -> Doc {
    RichDoc::concat(vec![left, doc, right])
}

fn annotate(core: &MetaCore, entity: &Field) -> Doc {
    let identifier = core
        .identifier(entity)
        .map_or(RichDoc::empty(), datom_value);

    RichDoc::concat(vec![
        identifier,
        punctuation("("),
        field(entity),
        punctuation(")"),
    ])
}

fn core_layout_value(core: &MetaCore, datom: &Datom) -> Doc {
    let attribute_type = core.meta_attribute_type(&datom.attribute).map(|d| &d.value);
    let reference_type = "3".into();
    if attribute_type == Some(&reference_type) {
        annotate(core, &datom.value)
    } else {
        surround(punctuation("\""), punctuation("\""), datom_value(datom))
    }
}

fn core_layout_attribute(core: &MetaCore, attr: (&Field, &HashSet<Datom>)) -> Doc {
    let (attr, values) = attr;
    RichDoc::concat(vec![
        RichDoc::linebreak(),
        RichDoc::group(RichDoc::nest(
            2,
            RichDoc::concat(vec![
                annotate(core, attr),
                whitespace(" "),
                punctuation("="),
                line(),
                RichDoc::concat(
                    values
                        .iter()
                        .map(|x| core_layout_value(core, x))
                        .intersperse(RichDoc::concat(vec![punctuation(","), line()]))
                        .collect(),
                ),
            ]),
        )),
    ])
}

pub fn core_layout_entity(core: &MetaCore, entity: &Field) -> Doc {
    let attributes = core
        .store
        .eav1(entity)
        .unwrap_or_else(|| panic!("{:?} has no attributes", entity));

    let type_ = core.meta_type(entity);

    let hide_attributes = hashset! {
        // identifier
        Field::from("0"),
        // type
        Field::from("5")
    };

    RichDoc::concat(vec![
        annotate(core, entity),
        whitespace(" "),
        punctuation(":"),
        whitespace(" "),
        type_.map_or_else(RichDoc::empty, |x| annotate(core, &x.value)),
        whitespace(" "),
        punctuation("{"),
        RichDoc::nest(
            2,
            RichDoc::concat(
                attributes
                    .iter()
                    .filter(|x| !hide_attributes.contains(x.0))
                    .sorted_by_key(|attr| attr.0)
                    .map(|attr| core_layout_attribute(&core, attr))
                    .collect(),
            ),
        ),
        RichDoc::linebreak(),
        punctuation("}"),
    ])
}

#[allow(dead_code)]
pub fn core_layout_entities(core: &MetaCore) -> Doc {
    let entities = core.store.entities().into_iter().sorted();
    RichDoc::concat(
        entities
            .map(|e| core_layout_entity(core, e))
            .intersperse(RichDoc::concat(vec![
                RichDoc::linebreak(),
                RichDoc::linebreak(),
            ]))
            .collect(),
    )
}

pub fn core_layout_datom(core: &MetaCore, datom: &Datom) -> Doc {
    RichDoc::nest(
        2,
        RichDoc::group(RichDoc::concat(vec![
            surround(punctuation("["), punctuation("]"), field(&datom.id)),
            line(),
            RichDoc::group(RichDoc::concat(vec![
                annotate(core, &datom.entity),
                punctuation("."),
                RichDoc::linebreak(),
                annotate(core, &datom.attribute),
            ])),
            whitespace(" "),
            punctuation("="),
            RichDoc::nest(
                2,
                RichDoc::concat(vec![line(), core_layout_value(core, datom)]),
            ),
        ])),
    )
}

#[allow(dead_code)]
pub fn core_layout_datoms(core: &MetaCore) -> Doc {
    let mut datoms: Vec<&Datom> = core.store.atoms().values().collect();
    datoms.sort_by_key(|d| &d.id);

    RichDoc::concat(
        datoms
            .iter()
            .map(|d| core_layout_datom(core, d))
            .intersperse(RichDoc::linebreak())
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

    RichDoc::concat(vec![
        text("language"),
        whitespace(" "),
        annotate(core, id),
        RichDoc::linebreak(),
        RichDoc::linebreak(),
        RichDoc::concat(
            entities
                .iter()
                .map(|e| core_layout_entity(core, &e.value))
                .intersperse(RichDoc::concat(vec![
                    RichDoc::linebreak(),
                    RichDoc::linebreak(),
                ]))
                .collect(),
        ),
    ])
}

pub fn core_layout_languages(core: &MetaCore) -> Doc {
    let type_id = "5".into();
    let language_id = "12".into();
    // TODO: core.get_of_type("12")
    let languages: Vec<&Datom> = core
        .store
        .ave2(&type_id, &language_id)
        .map_or_else(Vec::new, |x| x.iter().collect());

    RichDoc::concat(
        languages
            .iter()
            .map(|l| core_layout_language(core, &l.entity))
            .intersperse(RichDoc::linebreak())
            .collect(),
    )
}

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
    fn process_atom<'a>(
        x: &'a Datom,
        result: &'_ mut Vec<&'a Datom>,
        next: &HashMap<&'a Field, HashSet<&'a Datom>>,
    ) {
        result.push(x);
        if let Some(next_atoms) = next.get(&x.id) {
            for a in next_atoms.iter() {
                process_atom(a, result, next);
            }
        }
    }

    let mut result = Vec::new();
    for a in no_after.iter().sorted_by_key(|x| &x.id) {
        process_atom(a, &mut result, &next);
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
