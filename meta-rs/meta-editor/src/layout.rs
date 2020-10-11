use meta_pretty::{RichDoc, SimpleDoc, SimpleDocKind};
use meta_store::{Datom, Field};

use im::HashSet;
use unicode_segmentation::UnicodeSegmentation;

pub type Doc = RichDoc<EditorCellPayload>;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct TypeFilter(Option<HashSet<Field>>);

impl TypeFilter {
    pub fn new(types: Option<HashSet<Field>>) -> Self {
        Self(types)
    }

    pub fn from_types(types: HashSet<Field>) -> Self {
        Self(Some(types))
    }

    pub fn from_type(type_: Field) -> Self {
        Self::from_types(HashSet::unit(type_))
    }

    pub fn unfiltered() -> Self {
        Self(None)
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum CellClass {
    // order determines priority when selecting active cell
    Whitespace,
    Punctuation,
    NonEditable,
    Reference(Datom, ReferenceTarget, TypeFilter),
    Editable(Datom),
}

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum ReferenceTarget {
    Id,
    Entity,
    Attribute,
    Value,
}

impl ReferenceTarget {
    pub fn get_field<'a>(&self, datom: &'a Datom) -> &'a Field {
        match self {
            ReferenceTarget::Id => &datom.id,
            ReferenceTarget::Entity => &datom.entity,
            ReferenceTarget::Attribute => &datom.attribute,
            ReferenceTarget::Value => &datom.value,
        }
    }

    #[allow(dead_code)]
    pub fn get_field_mut<'a>(&self, datom: &'a mut Datom) -> &'a mut Field {
        match self {
            ReferenceTarget::Id => &mut datom.id,
            ReferenceTarget::Entity => &mut datom.entity,
            ReferenceTarget::Attribute => &mut datom.attribute,
            ReferenceTarget::Value => &mut datom.value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EditorCellPayload {
    pub text: CellText,
    pub class: CellClass,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CellText {
    Field(Field),
    Literal(&'static str),
}

// Specialize and re-export
pub fn concat(parts: Vec<Doc>) -> Doc {
    RichDoc::concat(parts)
}
pub fn empty() -> Doc {
    RichDoc::empty()
}
pub fn linebreak() -> Doc {
    RichDoc::linebreak()
}
pub fn group(doc: Doc) -> Doc {
    RichDoc::group(doc)
}
pub fn nest(width: usize, doc: Doc) -> Doc {
    RichDoc::nest(width, doc)
}

pub fn field(field: &Field) -> Doc {
    RichDoc::cell(
        str_length(field.as_ref()),
        EditorCellPayload {
            text: CellText::Field(field.clone()),
            class: CellClass::NonEditable,
        },
    )
}

pub fn datom_value(datom: &Datom) -> Doc {
    let field = &datom.value;
    RichDoc::cell(
        str_length(field.as_ref()),
        EditorCellPayload {
            text: CellText::Field(field.clone()),
            class: CellClass::Editable(datom.clone()),
        },
    )
}

pub fn datom_reference(
    datom: &Datom,
    target: ReferenceTarget,
    type_filter: TypeFilter,
    text: &Field,
) -> Doc {
    RichDoc::cell(
        str_length(text.as_ref()),
        EditorCellPayload {
            text: CellText::Field(text.clone()),
            class: CellClass::Reference(datom.clone(), target, type_filter),
        },
    )
}

pub fn punctuation(s: &'static str) -> Doc {
    literal(CellClass::Punctuation, s)
}

pub fn whitespace(s: &'static str) -> Doc {
    literal(CellClass::Whitespace, s)
}
pub fn line() -> Doc {
    RichDoc::line(meta_pretty::cell(
        1,
        EditorCellPayload {
            text: CellText::Literal(" "),
            class: CellClass::Whitespace,
        },
    ))
}

pub fn text(s: &'static str) -> Doc {
    literal(CellClass::NonEditable, s)
}

fn literal(class: CellClass, s: &'static str) -> Doc {
    RichDoc::cell(
        str_length(s),
        EditorCellPayload {
            text: CellText::Literal(s),
            class,
        },
    )
}

pub fn surround(left: Doc, right: Doc, doc: Doc) -> Doc {
    RichDoc::concat(vec![left, doc, right])
}

pub fn parentheses(doc: Doc) -> Doc {
    surround(punctuation("("), punctuation(")"), doc)
}

pub fn brackets(doc: Doc) -> Doc {
    surround(punctuation("["), punctuation("]"), doc)
}

pub fn quotes(doc: Doc) -> Doc {
    surround(punctuation("\""), punctuation("\""), doc)
}

impl AsRef<str> for CellText {
    fn as_ref(&self) -> &str {
        match self {
            CellText::Field(field) => field.as_ref(),
            CellText::Literal(s) => s,
        }
    }
}

pub fn cmp_priority(
    left: &SimpleDoc<EditorCellPayload>,
    right: &SimpleDoc<EditorCellPayload>,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    use SimpleDocKind::*;
    match (left.kind(), right.kind()) {
        (Linebreak { .. }, Linebreak { .. }) => Ordering::Equal,
        (Linebreak { .. }, Cell(..)) => Ordering::Less,
        (Cell(..), Linebreak { .. }) => Ordering::Greater,
        (Cell(left), Cell(right)) => left.payload.class.cmp(&right.payload.class),
    }
}

fn str_length(s: &str) -> usize {
    s.graphemes(true).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_length_unicode() {
        assert_eq!(6, str_length(&"привет"));
    }
}
