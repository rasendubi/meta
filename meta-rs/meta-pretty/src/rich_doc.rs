use std::{collections::HashMap, hash::Hash, rc::Rc};

use crate::path::{follow_path, pathify, Path, PathSegment};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Cell<T> {
    pub width: usize,
    pub payload: T,
}

#[inline]
pub fn cell<T>(width: usize, payload: T) -> Cell<T> {
    Cell { width, payload }
}

#[derive(Debug)]
pub struct RichDoc<T>(Rc<RichDocNode<T>>);

impl<T> Clone for RichDoc<T> {
    fn clone(&self) -> Self {
        RichDoc(self.0.clone())
    }
}

impl<T> RichDoc<T> {
    pub fn kind(&self) -> &RichDocKind<T> {
        &self.0.kind
    }

    pub fn key(&self) -> &Option<String> {
        &self.0.key
    }
}

impl<T> PartialEq for RichDoc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> Eq for RichDoc<T> {}

impl<T> Hash for RichDoc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.0, state)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RichDocNode<T> {
    kind: RichDocKind<T>,
    key: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RichDocKind<T> {
    Empty,
    Cell(Cell<T>),
    Line {
        alt: Option<Cell<T>>,
        // TODO: hard-break
    },
    Nest {
        nest_width: usize,
        doc: RichDoc<T>,
    },
    Concat {
        parts: Vec<RichDoc<T>>,
    },
    Group {
        doc: RichDoc<T>,
    },
}

// impl<T, M> RichDoc<T, M> {
//     pub fn map_meta<F, G, M2>(self, f: F, g: G) -> RichDoc<T, M2>
//     where
//         F: FnOnce(M) -> M2,
//         G: Fn(RichDocKind<T, M>) -> RichDocKind<T, M2>,
//     {
//         RichDoc {
//             meta: f(self.meta),
//             kind: g(self.kind),
//             key: self.key,
//         }
//     }
//
//     pub fn with_path(self) -> RichDoc<T, WithPath<M>> {
//         pathify(self, Vec::new())
//     }
// }

impl<T> RichDoc<T> {
    fn new(kind: RichDocKind<T>) -> Self {
        RichDoc(Rc::new(RichDocNode { kind, key: None }))
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
    pub fn nest(width: usize, doc: Self) -> Self {
        Self::new(RichDocKind::Nest {
            nest_width: width,
            doc,
        })
    }

    #[inline]
    pub fn concat(parts: Vec<Self>) -> Self {
        Self::new(RichDocKind::Concat { parts })
    }

    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let _empty = RichDoc::<&str>::empty();
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
