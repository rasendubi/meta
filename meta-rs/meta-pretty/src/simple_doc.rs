use crate::RichDoc;
use std::{hash::Hash, rc::Rc};

pub use crate::rich_doc::Cell;

#[derive(Debug)]
pub struct SimpleDoc<T>(Rc<SimpleDocNode<T>>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct SimpleDocNode<T> {
    pub kind: SimpleDocKind<T>,
    pub rich_doc: RichDoc<T>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum SimpleDocKind<T> {
    Cell(Cell<T>),
    Linebreak { indent_width: usize },
}

impl<T> Clone for SimpleDoc<T> {
    fn clone(&self) -> Self {
        SimpleDoc(self.0.clone())
    }
}

impl<T> PartialEq for SimpleDoc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for SimpleDoc<T> {}

impl<T> Hash for SimpleDoc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.0, state)
    }
}

impl<T> SimpleDoc<T> {
    fn new(node: SimpleDocNode<T>) -> Self {
        SimpleDoc(Rc::new(node))
    }

    pub fn kind(&self) -> &SimpleDocKind<T> {
        &self.0.kind
    }

    pub fn rich_doc(&self) -> &RichDoc<T> {
        &self.0.rich_doc
    }

    pub fn width(&self) -> usize {
        match self.kind() {
            SimpleDocKind::Cell(cell) => cell.width,
            SimpleDocKind::Linebreak { indent_width } => *indent_width,
        }
    }

    #[inline]
    pub(crate) fn linebreak(rich_doc: RichDoc<T>, indent_width: usize) -> Self {
        Self::new(SimpleDocNode {
            kind: SimpleDocKind::Linebreak { indent_width },
            rich_doc,
        })
    }

    #[inline]
    pub(crate) fn cell(rich_doc: RichDoc<T>, cell: Cell<T>) -> Self {
        Self::new(SimpleDocNode {
            kind: SimpleDocKind::Cell(cell),
            rich_doc,
        })
    }
}
