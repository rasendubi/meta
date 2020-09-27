pub use crate::rich_doc::Cell;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct SimpleDoc<T, M> {
    pub kind: SimpleDocKind<T>,
    pub meta: M,
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

    pub fn width(&self) -> usize {
        match &self.kind {
            SimpleDocKind::Cell(cell) => cell.width,
            SimpleDocKind::Linebreak { indent_width } => *indent_width,
        }
    }

    pub fn with_meta<M2>(self, meta: M2) -> SimpleDoc<T, M2> {
        SimpleDoc {
            meta,
            kind: self.kind,
        }
    }

    pub fn map_meta<F: FnOnce(M) -> M2, M2>(self, f: F) -> SimpleDoc<T, M2> {
        SimpleDoc {
            meta: f(self.meta),
            kind: self.kind,
        }
    }
}
