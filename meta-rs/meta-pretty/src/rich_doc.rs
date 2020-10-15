use std::{collections::HashMap, hash::Hash, ops::Deref, rc::Rc};

use crate::path::{follow_path, pathify, FollowPath, Path, PathSegment};

#[derive(Debug)]
pub struct RichDocRef<T, M = ()>(Rc<RichDoc<T, M>>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RichDoc<T, M = ()> {
    pub kind: RichDocKind<T, M>,
    pub key: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RichDocKind<T, M = ()> {
    Empty,
    Cell(Cell<T>),
    Line {
        /// Cell to draw when line is collapsed.
        alt: Option<Cell<T>>,
        // TODO: hard-break
    },
    Nest {
        nest_width: usize,
        doc: RichDocRef<T, M>,
    },
    Concat {
        parts: Vec<RichDocRef<T, M>>,
    },
    Group {
        doc: RichDocRef<T, M>,
    },
    Meta {
        doc: RichDocRef<T, M>,
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
    pub fn new(kind: RichDocKind<T, M>) -> Self {
        Self { kind, key: None }
    }

    pub fn with_key(mut self, key: Option<String>) -> Self {
        self.key = key;
        self
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
            doc: doc.into(),
        })
    }

    pub fn concat<I>(parts: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::new(RichDocKind::Concat {
            parts: parts.into_iter().map(|x| x.into()).collect(),
        })
    }

    pub fn group(doc: Self) -> Self {
        Self::new(RichDocKind::Group { doc: doc.into() })
    }

    pub fn meta(meta: M, doc: Self) -> Self {
        Self::new(RichDocKind::Meta {
            meta,
            doc: doc.into(),
        })
    }

    pub fn kind(&self) -> &RichDocKind<T, M> {
        &self.kind
    }

    pub fn key(&self) -> &Option<String> {
        &self.key
    }
}

impl<T, M> From<RichDoc<T, M>> for RichDocRef<T, M> {
    fn from(node: RichDoc<T, M>) -> Self {
        RichDocRef(Rc::new(node))
    }
}

impl<T, M> RichDocRef<T, M> {
    pub fn kind(&self) -> &RichDocKind<T, M> {
        &self.0.kind
    }

    pub fn key(&self) -> &Option<String> {
        &self.0.key
    }

    pub fn pathify(&self) -> HashMap<Self, Path> {
        let mut result = HashMap::new();
        pathify(&mut result, &self, Vec::new());
        result
    }

    pub fn follow_path<'a, 'b>(&'a self, path: &'b [PathSegment]) -> FollowPath<'a, 'b, T, M> {
        follow_path(self, path)
    }

    pub fn as_meta(&self) -> Option<&M> {
        if let RichDocKind::Meta { meta, doc: _doc } = self.kind() {
            Some(meta)
        } else {
            None
        }
    }
}

impl<T, M> From<Cell<T>> for RichDoc<T, M> {
    fn from(cell: Cell<T>) -> Self {
        RichDoc::cell(cell)
    }
}

impl<T, M> Deref for RichDocRef<T, M> {
    type Target = RichDoc<T, M>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, M> PartialEq for RichDocRef<T, M> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T, M> Eq for RichDocRef<T, M> {}

impl<T, M> Hash for RichDocRef<T, M> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.0, state)
    }
}

impl<T, M> Clone for RichDocRef<T, M> {
    fn clone(&self) -> Self {
        RichDocRef(self.0.clone())
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
