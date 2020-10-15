use std::{collections::HashMap, hash::Hash, rc::Rc};

use crate::path::{follow_path, pathify, Path, PathSegment};

#[derive(Debug)]
pub struct RichDoc<T, M = ()>(Rc<RichDocNode<T, M>>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct RichDocNode<T, M> {
    kind: RichDocKind<T, M>,
    key: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RichDocKind<T, M = ()> {
    Empty,
    Cell(Cell<T>),
    Line {
        /// Cell to draw on when line is collapsed.
        alt: Option<Cell<T>>,
        // TODO: hard-break
    },
    Nest {
        nest_width: usize,
        doc: RichDoc<T, M>,
    },
    Concat {
        parts: Vec<RichDoc<T, M>>,
    },
    Group {
        doc: RichDoc<T, M>,
    },
    Meta {
        doc: RichDoc<T, M>,
        meta: M,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Cell<T> {
    pub width: usize,
    pub payload: T,
}

impl<T> Cell<T> {
    pub fn new(width: usize, payload: T) -> Self {
        Self { width, payload }
    }
}

impl<T, M> RichDoc<T, M> {
    pub fn kind(&self) -> &RichDocKind<T, M> {
        &self.0.kind
    }

    pub fn key(&self) -> &Option<String> {
        &self.0.key
    }

    fn new(kind: RichDocKind<T, M>) -> Self {
        RichDoc(Rc::new(RichDocNode { kind, key: None }))
    }

    pub fn empty() -> Self {
        Self::new(RichDocKind::Empty)
    }

    pub fn cell(cell: Cell<T>) -> Self {
        Self::new(RichDocKind::Cell(cell))
    }

    pub fn line(alt: Cell<T>) -> Self {
        Self::new(RichDocKind::Line { alt: Some(alt) })
    }

    pub fn linebreak() -> Self {
        Self::new(RichDocKind::Line { alt: None })
    }

    pub fn nest(width: usize, doc: Self) -> Self {
        Self::new(RichDocKind::Nest {
            nest_width: width,
            doc,
        })
    }

    pub fn concat(parts: Vec<Self>) -> Self {
        Self::new(RichDocKind::Concat { parts })
    }

    pub fn group(doc: Self) -> Self {
        Self::new(RichDocKind::Group { doc })
    }

    pub fn pathify(&self) -> HashMap<Self, Path> {
        let mut result = HashMap::new();
        pathify(&mut result, &self, Vec::new());
        result
    }

    pub fn follow_path<'a, 'b>(
        &'a self,
        path: &'b [PathSegment],
    ) -> Result<&'a Self, (&'a Self, &'b [PathSegment])> {
        follow_path(self, path)
    }
}

impl<T, M> From<Cell<T>> for RichDoc<T, M> {
    fn from(cell: Cell<T>) -> Self {
        RichDoc::cell(cell)
    }
}

impl<T, M> PartialEq for RichDoc<T, M> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T, M> Eq for RichDoc<T, M> {}

impl<T, M> Hash for RichDoc<T, M> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.0, state)
    }
}

impl<T, M> Clone for RichDoc<T, M> {
    fn clone(&self) -> Self {
        RichDoc(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let _empty = RichDoc::<&str>::empty();
    }

    #[test]
    fn test_complex() {
        let _doc = RichDoc::<i32, ()>::group(RichDoc::nest(
            2,
            RichDoc::concat(vec![
                RichDoc::cell(Cell::new(2, 11)),
                RichDoc::empty(),
                RichDoc::line(Cell::new(2, 22)),
                RichDoc::linebreak(),
            ]),
        ));
    }
}
