use std::collections::HashSet;

use itertools::Itertools;
use maplit::hashset;

use meta_core::MetaCore;
use meta_pretty::RichDoc;
use meta_store::Field;

use crate::layout::{field, line, punctuation, EditorCellPayload};

type Doc = RichDoc<EditorCellPayload, ()>;

fn surround(left: Doc, right: Doc, doc: Doc) -> Doc {
    RichDoc::concat(vec![left, doc, right])
}

fn annotate(core: &MetaCore, entity: &Field) -> Doc {
    let identifier = core
        .identifier(entity)
        .map_or(RichDoc::empty(), |x| field(x));

    RichDoc::concat(vec![
        identifier,
        punctuation("("),
        field(entity),
        punctuation(")"),
    ])
}

fn core_layout_value(_core: &MetaCore, _attribute: &Field, value: &Field) -> Doc {
    surround(punctuation("\""), punctuation("\""), field(value))
}

fn core_layout_attribute(core: &MetaCore, attr: (&Field, &HashSet<Field>)) -> Doc {
    let (attr, values) = attr;
    RichDoc::concat(vec![
        RichDoc::linebreak(),
        RichDoc::group(RichDoc::nest(
            2,
            RichDoc::concat(vec![
                annotate(core, attr),
                punctuation(" "),
                punctuation("="),
                line(),
                RichDoc::concat(
                    values
                        .iter()
                        .map(|x| core_layout_value(core, attr, x))
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
        punctuation(" "),
        punctuation(":"),
        punctuation(" "),
        type_.map_or_else(RichDoc::empty, |x| annotate(core, x)),
        punctuation(" "),
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
