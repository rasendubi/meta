use std::collections::HashMap;

use crate::rich_doc::{RichDoc, RichDocKind};

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

pub(crate) fn follow_path<'a, 'b, T, M>(
    this: &'a RichDoc<T, M>,
    path: &'b [PathSegment],
) -> Result<&'a RichDoc<T, M>, (&'a RichDoc<T, M>, &'b [PathSegment])> {
    if path.is_empty() {
        return Ok(this);
    }

    match this.kind() {
        RichDocKind::Nest { doc, .. } if path[0] == PathSegment::Nest => {
            return follow_path(doc, &path[1..]);
        }
        RichDocKind::Concat { parts } => match &path[0] {
            PathSegment::Index(_, Some(s)) => {
                if let Some(doc) = parts.iter().find(|x| x.key().as_ref() == Some(s)) {
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
