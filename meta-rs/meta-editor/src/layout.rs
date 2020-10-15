use im::HashSet;
use unicode_segmentation::UnicodeSegmentation;

use meta_pretty::{Cell, RichDoc, RichDocRef, SimpleDoc, SimpleDocKind};
use meta_store::{Datom, Field};

use crate::key::KeyHandler;

pub type Doc = RichDocRef<EditorCellPayload, Box<dyn KeyHandler>>;
pub type RDoc = RichDoc<EditorCellPayload, Box<dyn KeyHandler>>;
pub type SDoc = SimpleDoc<EditorCellPayload, Box<dyn KeyHandler>>;

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

    pub fn filter(&self) -> &Option<HashSet<Field>> {
        &self.0
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

pub fn with_key_handler(key_handler: Box<dyn KeyHandler>, doc: RDoc) -> RDoc {
    RichDoc::meta(key_handler, doc)
}

// Specialize and re-export
pub fn concat(parts: Vec<RDoc>) -> RDoc {
    RichDoc::concat(parts)
}
pub fn empty() -> RDoc {
    RichDoc::empty()
}
pub fn linebreak() -> RDoc {
    RichDoc::linebreak()
}
pub fn group(doc: RDoc) -> RDoc {
    RichDoc::group(doc)
}
pub fn nest(width: usize, doc: RDoc) -> RDoc {
    RichDoc::nest(width, doc)
}

pub fn field(field: &Field) -> RDoc {
    RichDoc::cell(Cell::new(
        str_length(field.as_ref()),
        EditorCellPayload {
            text: CellText::Field(field.clone()),
            class: CellClass::NonEditable,
        },
    ))
}

pub fn datom_value(datom: &Datom) -> RDoc {
    let field = &datom.value;
    RichDoc::cell(Cell::new(
        str_length(field.as_ref()),
        EditorCellPayload {
            text: CellText::Field(field.clone()),
            class: CellClass::Editable(datom.clone()),
        },
    ))
}

pub fn datom_reference(
    datom: &Datom,
    target: ReferenceTarget,
    type_filter: TypeFilter,
    text: &Field,
) -> RDoc {
    RichDoc::cell(Cell::new(
        str_length(text.as_ref()),
        EditorCellPayload {
            text: CellText::Field(text.clone()),
            class: CellClass::Reference(datom.clone(), target, type_filter),
        },
    ))
}

pub fn punctuation(s: &'static str) -> RDoc {
    literal(CellClass::Punctuation, s)
}

pub fn whitespace(s: &'static str) -> RDoc {
    literal(CellClass::Whitespace, s)
}
pub fn line() -> RDoc {
    RichDoc::line(literal_cell(CellClass::Whitespace, " "))
}

pub fn text(s: &'static str) -> RDoc {
    literal(CellClass::NonEditable, s)
}

fn literal(class: CellClass, s: &'static str) -> RDoc {
    literal_cell(class, s).into()
}

fn literal_cell(class: CellClass, s: &'static str) -> Cell<EditorCellPayload> {
    Cell::new(
        str_length(s),
        EditorCellPayload {
            text: CellText::Literal(s),
            class,
        },
    )
}

pub fn surround(left: RDoc, right: RDoc, doc: RDoc) -> RDoc {
    RichDoc::concat(vec![left, doc, right])
}

pub fn parentheses(doc: RDoc) -> RDoc {
    surround(punctuation("("), punctuation(")"), doc)
}

pub fn brackets(doc: RDoc) -> RDoc {
    surround(punctuation("["), punctuation("]"), doc)
}

pub fn quotes(doc: RDoc) -> RDoc {
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

pub fn cmp_priority<M>(
    left: &SimpleDoc<EditorCellPayload, M>,
    right: &SimpleDoc<EditorCellPayload, M>,
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
