pub use crate::rich_doc::Cell;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum SimpleDoc<T> {
    Cell(Cell<T>),
    Linebreak { indent_width: usize },
}

impl<T> SimpleDoc<T> {
    #[inline]
    pub fn from_cell(cell: Cell<T>) -> Self {
        SimpleDoc::Cell(cell)
    }

    #[inline]
    pub fn linebreak(indent_width: usize) -> Self {
        SimpleDoc::Linebreak { indent_width }
    }
}

impl<T> SimpleDoc<T>
where
    T: Clone,
{
    #[inline]
    pub fn cell(cell: &Cell<T>) -> Self {
        SimpleDoc::Cell(cell.clone())
    }
}
