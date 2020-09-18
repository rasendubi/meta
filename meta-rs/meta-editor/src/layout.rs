use meta_pretty::RichDoc;
use meta_store::Field;

pub type Doc = RichDoc<EditorCellPayload, ()>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EditorCellPayload {
    pub text: CellText,
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
        },
    )
}

pub fn punctuation(s: &'static str) -> Doc {
    RichDoc::cell(
        s.len(),
        EditorCellPayload {
            text: CellText::Literal(s),
        },
    )
}

pub fn line() -> Doc {
    RichDoc::line(meta_pretty::cell(
        1,
        EditorCellPayload {
            text: CellText::Literal(" "),
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
