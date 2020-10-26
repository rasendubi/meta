use std::{cmp::Ordering, collections::HashMap};

use itertools::Either;

use meta_pretty::{Path, SimpleDoc, SimpleDocKind};

use crate::editor::{CellPosition, CursorPosition};
use crate::layout::{cmp_priority, Doc, SDoc};

/// `DocView` encapsulates information derived from `Doc` such as its pretty-printed layout, paths
/// of the nodes, and positions of the cells.
pub(crate) struct DocView {
    rich_doc: Doc,

    // derived fields
    layout: Vec<Vec<SDoc>>,
    paths: HashMap<Doc, Path>,
    positions: HashMap<SDoc, CellPosition>,
}

impl DocView {
    pub fn new(rich_doc: Doc) -> Self {
        let sdoc = meta_pretty::layout(&rich_doc, 80);
        let layout = layout_to_2d(&sdoc);

        let paths = rich_doc.pathify();
        let positions = enumerate(&layout);

        Self {
            rich_doc,
            layout,
            paths,
            positions,
        }
    }

    pub fn doc(&self) -> &Doc {
        &self.rich_doc
    }

    pub fn layout(&self) -> &Vec<Vec<SDoc>> {
        &self.layout
    }

    /// Find SDoc corresponding to the Doc.
    ///
    /// Complexity: O(n) where n is the size of the DocView.
    pub fn rdoc_to_sdoc(&self, rdoc: &Doc) -> Option<&SDoc> {
        for row in self.layout.iter() {
            for sdoc in row.iter() {
                if sdoc.rich_doc() == rdoc {
                    return Some(sdoc);
                }
            }
        }

        None
    }

    pub fn get_node_path(&self, doc: &Doc) -> Option<&Path> {
        self.paths.get(doc)
    }

    pub fn get_sdoc_position(&self, sdoc: &SDoc) -> Option<&CellPosition> {
        self.positions.get(sdoc)
    }

    pub fn find_sdoc(&self, sdoc: &SDoc) -> Option<(usize, usize)> {
        for (i, row) in self.layout.iter().enumerate() {
            for (j, s) in row.iter().enumerate() {
                if s == sdoc {
                    return Some((i, j));
                }
            }
        }

        None
    }

    pub fn cell_position_to_cursor(&self, pos: CellPosition) -> Option<CursorPosition> {
        let CellPosition { row, col } = pos;
        self.layout.get(row).and_then(|r| {
            let m: Result<
                /* out of bound */ Option<&SDoc>,
                Either</* between */ (SDoc, SDoc), /* inside */ CursorPosition>,
            > = r
                .iter()
                .try_fold(None, |acc: Option<&SimpleDoc<_, _>>, cell| {
                    let left = self.get_sdoc_position(cell).unwrap().col;
                    let right = left + cell.width();
                    if col < left || right <= col {
                        Ok(Some(cell))
                    } else if left == col {
                        Err(match acc {
                            None => Either::Right(CursorPosition {
                                sdoc: cell.clone(),
                                offset: col - left,
                            }),
                            Some(prev) => Either::Left((prev.clone(), cell.clone())),
                        })
                    } else {
                        // strictly inside cell
                        Err(Either::Right(CursorPosition {
                            sdoc: cell.clone(),
                            offset: col - left,
                        }))
                    }
                });
            match m {
                // inside
                Err(Either::Right(position)) => Some(position),
                // between
                Err(Either::Left((left, right))) => Some(resolve_cursor_priority(left, right)),
                // out of bound
                Ok(mcell) => mcell.map(|cell| CursorPosition {
                    sdoc: cell.clone(),
                    offset: cell.width(),
                }),
            }
        })
    }
}

fn resolve_cursor_priority(left: SDoc, right: SDoc) -> CursorPosition {
    match cmp_priority(&left, &right) {
        Ordering::Less | Ordering::Equal => CursorPosition {
            sdoc: right,
            offset: 0,
        },
        Ordering::Greater => CursorPosition {
            offset: left.width(),
            sdoc: left,
        },
    }
}

fn enumerate<T, M>(layout: &[Vec<SimpleDoc<T, M>>]) -> HashMap<SimpleDoc<T, M>, CellPosition> {
    let mut result = HashMap::new();

    for (row_id, row) in layout.iter().enumerate() {
        let mut column = 0;
        for cell in row.iter() {
            result.insert(cell.clone(), CellPosition::new(row_id, column));
            column += cell.width();
        }
    }

    result
}

fn layout_to_2d<T, M>(layout: &[SimpleDoc<T, M>]) -> Vec<Vec<SimpleDoc<T, M>>> {
    let mut result = vec![Vec::new()];

    for cell in layout.iter() {
        if let SimpleDocKind::Linebreak { .. } = cell.kind() {
            result.push(Vec::new());
        }

        result.last_mut().unwrap().push(cell.clone());
    }

    result
}

#[allow(dead_code)]
fn simple_doc_to_string(sdoc: &[SDoc]) -> String {
    let mut out = String::new();

    for doc in sdoc {
        match doc.kind() {
            SimpleDocKind::Linebreak { indent_width } => {
                out.reserve(indent_width + 1);
                out.push('\n');
                for _ in 0..*indent_width {
                    out.push(' ');
                }
            }
            SimpleDocKind::Cell(cell) => {
                out.push_str(cell.payload.text.as_ref());
            }
        }
    }

    out
}
