use crate::rich_doc::{RichDoc, RichDocKind};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WithPath<M = ()> {
    meta: M,
    path: Vec<String>,
}

pub(crate) fn pathify<T, M>(
    doc: RichDoc<T, M>,
    path_prefix: Vec<String>,
) -> RichDoc<T, WithPath<M>> {
    doc.map_meta(
        |meta| WithPath::new(meta, path_prefix.clone()),
        |kind| match kind {
            RichDocKind::Empty => RichDocKind::Empty,
            RichDocKind::Cell(cell) => RichDocKind::Cell(cell),
            RichDocKind::Line { alt } => RichDocKind::Line { alt },
            RichDocKind::Nest { nest_width, doc } => {
                let mut path = path_prefix.clone();
                path.push("nest".to_string());

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
                        let key = part
                            .key
                            .as_ref()
                            .map_or_else(|| i.to_string(), |key| key.clone());
                        let mut path = path_prefix.clone();
                        path.push(key);
                        pathify(part, path)
                    })
                    .collect();
                RichDocKind::Concat { parts }
            }
            RichDocKind::Group { doc } => {
                let mut path = path_prefix.clone();
                path.push("group".to_string());

                RichDocKind::Group {
                    doc: Box::new(pathify(*doc, path)),
                }
            }
        },
    )
}

impl<M> WithPath<M> {
    fn new(meta: M, path: Vec<String>) -> Self {
        WithPath { meta, path }
    }

    pub fn path(&self) -> &Vec<String> {
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
