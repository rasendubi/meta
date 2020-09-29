use im::HashSet;

use itertools::Itertools;
use maplit::hashset;

use meta_core::MetaCore;
use meta_pretty::RichDoc;
use meta_store::{Datom, Field};

use crate::layout::{datom_value, field, line, punctuation, whitespace, EditorCellPayload};

type Doc = RichDoc<EditorCellPayload, ()>;

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
