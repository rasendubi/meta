use std::collections::HashMap;

use crate::rich_doc::{RichDoc, RichDocKind};

pub enum FollowPath<'a, 'b, T, M> {
    Done,
    Ok(&'a RichDoc<T, M>, &'b [PathSegment]),
    Error(&'a RichDoc<T, M>, &'b [PathSegment]),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum PathSegment {
    Nest,
    Group,
    Meta,
    /// A child of `RichDoc::Concat`. Stores index and optional key.
    Index(usize, Option<String>),
}

// TODO: consider using im::Vector for Path as it allows O(1) clone and structural sharing.
pub type Path = Vec<PathSegment>;

pub(crate) fn pathify<T, M>(
    result: &mut HashMap<RichDoc<T, M>, Path>,
    doc: &RichDoc<T, M>,
    path: Vec<PathSegment>,
) {
    match doc.kind() {
        RichDocKind::Nest { doc, .. } => {
            let mut path = path.clone();
            path.push(PathSegment::Nest);

            pathify(result, doc, path);
        }
        RichDocKind::Concat { parts } => {
            for (i, part) in parts.iter().enumerate() {
                let mut path = path.clone();
                path.push(PathSegment::Index(i, part.key().as_ref().cloned()));
                pathify(result, part, path)
            }
        }
        RichDocKind::Group { doc } => {
            let mut path = path.clone();
            path.push(PathSegment::Group);
            pathify(result, doc, path);
        }
        RichDocKind::Meta { doc, meta: _meta } => {
            let mut path = path.clone();
            path.push(PathSegment::Meta);
            pathify(result, doc, path);
        }
        RichDocKind::Empty | RichDocKind::Cell(_) | RichDocKind::Line { .. } => {}
    };

    result.insert(doc.clone(), path);
}

impl<'a, 'b, T, M> Iterator for FollowPath<'a, 'b, T, M> {
    #[allow(clippy::type_complexity)]
    type Item = Result<&'a RichDoc<T, M>, (&'a RichDoc<T, M>, &'b [PathSegment])>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut this = FollowPath::Done;
        std::mem::swap(self, &mut this);

        match this {
            FollowPath::Done => None,
            FollowPath::Ok(doc, path) => {
                *self = if path.is_empty() {
                    FollowPath::Done
                } else if let Some(next) = follow_segment(&doc, &path[0]) {
                    FollowPath::Ok(next, &path[1..])
                } else {
                    FollowPath::Error(doc, path)
                };

                Some(Ok(doc))
            }
            FollowPath::Error(doc, path) => {
                *self = FollowPath::Done;

                Some(Err((doc, path)))
            }
        }
    }
}

fn follow_segment<'a, 'b, T, M>(
    doc: &'a RichDoc<T, M>,
    segment: &'b PathSegment,
) -> Option<&'a RichDoc<T, M>> {
    match doc.kind() {
        RichDocKind::Nest { doc, .. } if *segment == PathSegment::Nest => {
            return Some(doc);
        }
        RichDocKind::Concat { parts } => match segment {
            PathSegment::Index(_, Some(s)) => {
                if let Some(doc) = parts.iter().find(|x| x.key().as_ref() == Some(s)) {
                    return Some(doc);
                }
            }
            PathSegment::Index(i, None) => {
                if let Some(doc) = parts.get(*i) {
                    return Some(doc);
                }
            }
            _ => {}
        },
        RichDocKind::Group { doc } if *segment == PathSegment::Group => {
            return Some(doc);
        }
        RichDocKind::Meta { doc, meta: _meta } if *segment == PathSegment::Meta => {
            return Some(doc);
        }
        _ => {}
    }

    None
}

pub(crate) fn follow_path<'a, 'b, T, M>(
    doc: &'a RichDoc<T, M>,
    path: &'b [PathSegment],
) -> FollowPath<'a, 'b, T, M> {
    FollowPath::Ok(doc, path)
}
