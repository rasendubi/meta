use crate::rich_doc::{RichDoc, RichDocKind};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum PathSegment {
    Nest,
    Group,
    /// A child of concat cell. Stores index and optional key.
    Index(usize, Option<String>),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WithPath<M = ()> {
    pub meta: M,
    pub path: Vec<PathSegment>,
}

pub(crate) fn pathify<T, M>(
    doc: RichDoc<T, M>,
    path_prefix: Vec<PathSegment>,
) -> RichDoc<T, WithPath<M>> {
    doc.map_meta(
        |meta| WithPath::new(meta, path_prefix.clone()),
        |kind| match kind {
            RichDocKind::Empty => RichDocKind::Empty,
            RichDocKind::Cell(cell) => RichDocKind::Cell(cell),
            RichDocKind::Line { alt } => RichDocKind::Line { alt },
            RichDocKind::Nest { nest_width, doc } => {
                let mut path = path_prefix.clone();
                path.push(PathSegment::Nest);

                RichDocKind::Nest {
                    nest_width,
                    doc: Box::new(pathify(*doc, path)),
                }
            }
            RichDocKind::Concat { parts } => {
                let parts = parts
                    .into_iter()
                    .enumerate()
                    .map(|(i, part)| {
                        let key = part.key.as_ref().cloned();

                        let mut path = path_prefix.clone();
                        path.push(PathSegment::Index(i, key));
                        pathify(part, path)
                    })
                    .collect();
                RichDocKind::Concat { parts }
            }
            RichDocKind::Group { doc } => {
                let mut path = path_prefix.clone();
                path.push(PathSegment::Group);

                RichDocKind::Group {
                    doc: Box::new(pathify(*doc, path)),
                }
            }
        },
    )
}

#[allow(clippy::type_complexity)]
pub(crate) fn follow_path<'a, 'b, T, M>(
    this: &'a RichDoc<T, M>,
    path: &'b [PathSegment],
) -> Result<&'a RichDoc<T, M>, (&'a RichDoc<T, M>, &'b [PathSegment])> {
    if path.is_empty() {
        return Ok(this);
    }

    match &this.kind {
        RichDocKind::Nest { doc, .. } if path[0] == PathSegment::Nest => {
            return follow_path(doc, &path[1..]);
        }
        RichDocKind::Concat { parts } => match &path[0] {
            PathSegment::Index(_, Some(s)) => {
                if let Some(doc) = parts.iter().find(|x| x.key.as_ref() == Some(s)) {
                    return follow_path(doc, &path[1..]);
                }
            }
            PathSegment::Index(i, None) => {
                if let Some(doc) = parts.get(*i) {
                    return follow_path(doc, &path[1..]);
                }
            }
            _ => {}
        },
        RichDocKind::Group { doc } if path[0] == PathSegment::Group => {
            return follow_path(doc, &path[1..]);
        }
        _ => {}
    }

    Err((this, path))
}

impl<M> WithPath<M> {
    fn new(meta: M, path: Vec<PathSegment>) -> Self {
        WithPath { meta, path }
    }

    pub fn path(&self) -> &Vec<PathSegment> {
        &self.path
    }
}

impl<M> std::convert::AsRef<M> for WithPath<M> {
    fn as_ref(&self) -> &M {
        &self.meta
    }
}

impl<M> std::convert::AsMut<M> for WithPath<M> {
    fn as_mut(&mut self) -> &mut M {
        &mut self.meta
    }
}
