use meta_pretty::{RichDoc, SimpleDoc, SimpleDocKind};
use meta_store::{Datom, Field};

pub type Doc = RichDoc<EditorCellPayload, ()>;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum CellClass {
    // order determines priority when selecting active cell
    Whitespace,
    Punctuation,
    NonEditable,
    Editable,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EditorCellPayload {
    pub text: CellText,
    pub class: CellClass,
    pub datom: Option<Datom>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CellText {
    Field(Field),
    Literal(&'static str),
}

pub fn field(field: &Field) -> Doc {
    RichDoc::cell(
        field.as_ref().len(),
        EditorCellPayload {
            text: CellText::Field(field.clone()),
            class: CellClass::NonEditable,
            datom: None,
        },
    )
}

pub fn datom(datom: Datom) -> Doc {
    let field = &datom.value;
    RichDoc::cell(
        field.as_ref().len(),
        EditorCellPayload {
            text: CellText::Field(field.clone()),
            class: CellClass::Editable,
            datom: Some(datom),
        },
    )
}

pub fn punctuation(s: &'static str) -> Doc {
    RichDoc::cell(
        s.len(),
        EditorCellPayload {
            text: CellText::Literal(s),
            class: CellClass::Punctuation,
            datom: None,
        },
    )
}

pub fn whitespace(s: &'static str) -> Doc {
    RichDoc::cell(
        s.len(),
        EditorCellPayload {
            text: CellText::Literal(s),
            class: CellClass::Whitespace,
            datom: None,
        },
    )
}

pub fn line() -> Doc {
    RichDoc::line(meta_pretty::cell(
        1,
        EditorCellPayload {
            text: CellText::Literal(" "),
            class: CellClass::Whitespace,
            datom: None,
        },
    ))
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
    match (&left.kind, &right.kind) {
        (Linebreak { .. }, Linebreak { .. }) => Ordering::Equal,
        (Linebreak { .. }, Cell(..)) => Ordering::Less,
        (Cell(..), Linebreak { .. }) => Ordering::Greater,
        (Cell(left), Cell(right)) => left.payload.class.cmp(&right.payload.class),
    }
}
