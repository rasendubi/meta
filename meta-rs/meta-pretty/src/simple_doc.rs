pub use crate::rich_doc::Cell;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SimpleDoc<'a, T> {
    Cell(&'a Cell<T>),
    Linebreak { indent_width: usize },
}

impl<'a, T> SimpleDoc<'a, T> {
    #[inline]
    pub fn cell(cell: &'a Cell<T>) -> Self {
        SimpleDoc::Cell(cell)
    }

    #[inline]
    pub fn linebreak(indent_width: usize) -> Self {
        SimpleDoc::Linebreak { indent_width }
    }
}
