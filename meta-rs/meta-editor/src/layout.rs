use meta_pretty::{RichDoc, SimpleDoc, SimpleDocKind};
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

pub fn simple_doc_to_string(sdoc: &[SimpleDoc<EditorCellPayload, ()>]) -> String {
    let mut out = String::new();

    for doc in sdoc {
        match &doc.kind {
            SimpleDocKind::Linebreak { indent_width } => {
                out.reserve(indent_width + 1);
                out.push('\r');
                for _ in 0..*indent_width {
                    out.push(' ');
                }
            }
            SimpleDocKind::Cell(cell) => {
                out.push_str(cell.payload.text.as_ref());
            }
        }
    }

    out
}

impl AsRef<str> for CellText {
    fn as_ref(&self) -> &str {
        match self {
            CellText::Field(field) => field.as_ref(),
            CellText::Literal(s) => s,
        }
    }
}
