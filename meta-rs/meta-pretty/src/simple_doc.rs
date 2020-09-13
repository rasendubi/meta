pub use crate::rich_doc::Cell;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct SimpleDoc<T, M> {
    pub meta: M,
    pub kind: SimpleDocKind<T>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum SimpleDocKind<T> {
    Cell(Cell<T>),
    Linebreak { indent_width: usize },
}

impl<T, M> SimpleDoc<T, M> {
    #[inline]
    pub(crate) fn linebreak(meta: M, indent_width: usize) -> Self {
        SimpleDoc {
            meta,
            kind: SimpleDocKind::Linebreak { indent_width },
        }
    }

    #[inline]
    pub(crate) fn cell(meta: M, cell: Cell<T>) -> Self {
        SimpleDoc {
            meta,
            kind: SimpleDocKind::Cell(cell),
        }
    }
}
