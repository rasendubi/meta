use crate::path::{follow_path, pathify, PathSegment, WithPath};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Cell<T> {
    pub width: usize,
    pub payload: T,
}

#[inline]
pub fn cell<T>(width: usize, payload: T) -> Cell<T> {
    Cell { width, payload }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RichDoc<T, M> {
    pub meta: M,
    pub kind: RichDocKind<T, M>,
    pub key: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RichDocKind<T, M> {
    Empty,
    Cell(Cell<T>),
    Line {
        alt: Option<Cell<T>>,
        // TODO: hard-break
    },
    Nest {
        nest_width: usize,
        doc: Box<RichDoc<T, M>>,
    },
    Concat {
        parts: Vec<RichDoc<T, M>>,
    },
    Group {
        doc: Box<RichDoc<T, M>>,
    },
}

impl<T, M> RichDoc<T, M> {
    pub fn map_meta<F: FnOnce(M) -> M2, G: Fn(RichDocKind<T, M>) -> RichDocKind<T, M2>, M2>(
        self,
        f: F,
        g: G,
    ) -> RichDoc<T, M2> {
        RichDoc {
            meta: f(self.meta),
            kind: g(self.kind),
            key: self.key,
        }
    }

    pub fn with_path(self) -> RichDoc<T, WithPath<M>> {
        pathify(self, Vec::new())
    }
}

impl<T, M> RichDoc<T, WithPath<M>> {
    pub fn follow_path<'a, 'b>(
        &'a self,
        path: &'b [PathSegment],
    ) -> Result<&'a Self, (&'a Self, &'b [PathSegment])> {
        follow_path(self, path)
    }
}

impl<T> RichDoc<T, ()> {
    fn new(kind: RichDocKind<T, ()>) -> Self {
        RichDoc {
            meta: (),
            kind,
            key: None,
        }
    }

    #[inline]
    pub fn empty() -> Self {
        Self::new(RichDocKind::Empty)
    }

    #[inline]
    pub fn cell(width: usize, payload: T) -> Self {
        Self::new(RichDocKind::Cell(Cell { width, payload }))
    }

    #[inline]
    pub fn line(alt: Cell<T>) -> Self {
        Self::new(RichDocKind::Line { alt: Some(alt) })
    }

    #[inline]
    pub fn linebreak() -> Self {
        Self::new(RichDocKind::Line { alt: None })
    }

    #[inline]
    pub fn nest(width: usize, doc: RichDoc<T, ()>) -> Self {
        Self::new(RichDocKind::Nest {
            nest_width: width,
            doc: Box::new(doc),
        })
    }

    #[inline]
    pub fn concat(parts: Vec<RichDoc<T, ()>>) -> Self {
        Self::new(RichDocKind::Concat { parts })
    }

    #[inline]
    pub fn group(doc: RichDoc<T, ()>) -> Self {
        Self::new(RichDocKind::Group { doc: Box::new(doc) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let _empty = RichDoc::<&str, ()>::empty();
    }

    #[test]
    fn test_complex() {
        let _doc = RichDoc::group(RichDoc::nest(
            2,
            RichDoc::concat(vec![
                RichDoc::cell(2, 11),
                RichDoc::empty(),
                RichDoc::line(cell(2, 22)),
                RichDoc::linebreak(),
            ]),
        ));
    }
}
