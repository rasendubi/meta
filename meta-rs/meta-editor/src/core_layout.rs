use im::HashSet;

use itertools::Itertools;
use maplit::hashset;

use meta_core::MetaCore;
use meta_pretty::RichDoc;
use meta_store::{Datom, Field};

use crate::layout::{datom_value, field, line, punctuation, text, whitespace, EditorCellPayload};

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
