use std::{fmt::Debug, hash::Hash, rc::Rc};

use crate::RichDocRef;

pub use crate::rich_doc::Cell;

pub struct SimpleDoc<T, M = ()>(Rc<SimpleDocNode<T, M>>);

#[derive(PartialEq, Eq, Hash, Clone)]
struct SimpleDocNode<T, M> {
    pub kind: SimpleDocKind<T>,
    pub rich_doc: RichDocRef<T, M>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum SimpleDocKind<T> {
    Cell(Cell<T>),
    Linebreak { indent_width: usize },
}

impl<T, M> Debug for SimpleDoc<T, M>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SimpleDoc").field(&self.0).finish()
    }
}

impl<T, M> Debug for SimpleDocNode<T, M>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleDocNode")
            .field("kind", &self.kind)
            .finish()
    }
}

impl<T, M> SimpleDoc<T, M> {
    fn new(node: SimpleDocNode<T, M>) -> Self {
        SimpleDoc(Rc::new(node))
    }

    pub fn kind(&self) -> &SimpleDocKind<T> {
        &self.0.kind
    }

    pub fn rich_doc(&self) -> &RichDocRef<T, M> {
        &self.0.rich_doc
    }

    pub fn width(&self) -> usize {
        match self.kind() {
            SimpleDocKind::Cell(cell) => cell.width,
            SimpleDocKind::Linebreak { indent_width } => *indent_width,
        }
    }

    #[inline]
    pub(crate) fn linebreak(rich_doc: RichDocRef<T, M>, indent_width: usize) -> Self {
        Self::new(SimpleDocNode {
            kind: SimpleDocKind::Linebreak { indent_width },
            rich_doc,
        })
    }

    #[inline]
    pub(crate) fn cell(rich_doc: RichDocRef<T, M>, cell: Cell<T>) -> Self {
        Self::new(SimpleDocNode {
            kind: SimpleDocKind::Cell(cell),
            rich_doc,
        })
    }
}

impl<T, M> PartialEq for SimpleDoc<T, M> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T, M> Eq for SimpleDoc<T, M> {}

impl<T, M> Hash for SimpleDoc<T, M> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.0, state)
    }
}

impl<T, M> Clone for SimpleDoc<T, M> {
    fn clone(&self) -> Self {
        SimpleDoc(self.0.clone())
    }
}
