use log::Level::Trace;
use log::{debug, log_enabled, trace};

use druid_shell::kurbo::{Insets, Rect, Size};
use druid_shell::{piet::Color, KeyCode};
use meta_gui::{Constraint, Direction, Event, EventType, GuiContext, Inset, Layout, List};

use crate::cell_widget::CellWidget;
use crate::core_layout::core_layout_entities;
use crate::layout::{cmp_priority, EditorCellPayload};
use meta_core::MetaCore;
use meta_pretty::{SimpleDoc, SimpleDocKind, WithPath};
use meta_store::MetaStore;
use std::cmp::Ordering;

pub type LayoutMeta = WithEnumerate<WithPath<()>>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CursorPosition<M> {
    Inside {
        cell: SimpleDoc<EditorCellPayload, M>,
        offset: usize,
    },
    Between(
        SimpleDoc<EditorCellPayload, M>,
        SimpleDoc<EditorCellPayload, M>,
    ),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellPosition {
    row: usize,
    col: usize,
}

pub struct Editor {
    layout: Vec<Vec<SimpleDoc<EditorCellPayload, LayoutMeta>>>,
    cursor: Option<CursorPosition<LayoutMeta>>,
    pos: CellPosition,
}

impl Editor {
    pub fn new(store: MetaStore) -> Self {
        let core = MetaCore::new(&store);
        let rich_doc = core_layout_entities(&core).with_path();
        let sdoc = meta_pretty::layout(&rich_doc, 80);

        if log_enabled!(Trace) {
            trace!("layout:\n{}", simple_doc_to_string(&sdoc));
        }

        let layout = enumerate(layout_to_2d(sdoc));
        let pos = CellPosition { row: 0, col: 0 };
        let cursor = Editor::cell_position_to_cursor(&layout, &pos);

        Editor {
            layout,
            pos,
            cursor,
        }
    }

    fn move_cursor(&mut self, drow: isize, dcol: isize) {
        let pos = CellPosition {
            row: (self.pos.row as isize + drow) as usize,
            col: (self.pos.col as isize + dcol) as usize,
        };
        let cursor = Self::cell_position_to_cursor(&self.layout, &pos);

        self.pos = pos;
        self.cursor = cursor;
    }

    fn cell_position_to_cursor(
        layout: &[Vec<SimpleDoc<EditorCellPayload, LayoutMeta>>],
        pos: &CellPosition,
    ) -> Option<CursorPosition<LayoutMeta>> {
        let CellPosition { row, col } = pos;
        layout
            .get(*row)
            .and_then(|r| {
                let m = r
                    .iter()
                    .try_fold(None, |acc: Option<&SimpleDoc<_, _>>, cell| {
                        let left = cell.meta.pos.col;
                        let right = left + cell.width();
                        if *col < left || right <= *col {
                            Ok(Some(cell))
                        } else if left == *col {
                            Err(match acc {
                                None => CursorPosition::Inside {
                                    cell: cell.clone(),
                                    offset: col - left,
                                },
                                Some(prev) => CursorPosition::Between(prev.clone(), cell.clone()),
                            })
                        } else {
                            // strictly inside cell
                            Err(CursorPosition::Inside {
                                cell: cell.clone(),
                                offset: col - left,
                            })
                        }
                    });
                match m {
                    Err(position) => Some(position),
                    Ok(mcell) => mcell.map(|cell| CursorPosition::Inside {
                        cell: cell.clone(),
                        offset: cell.width(),
                    }),
                }
            })
            .map(Self::resolve_cursor_priority)
    }

    fn resolve_cursor_priority<T>(cursor: CursorPosition<T>) -> CursorPosition<T> {
        match cursor {
            CursorPosition::Inside { .. } => cursor,
            CursorPosition::Between(left, right) => match cmp_priority(&left, &right) {
                Ordering::Less | Ordering::Equal => CursorPosition::Inside {
                    cell: right,
                    offset: 0,
                },
                Ordering::Greater => CursorPosition::Inside {
                    offset: left.width(),
                    cell: left,
                },
            },
        }
    }
}

impl Layout for Editor {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        ctx.clear(Color::WHITE);

        let cursor = &self.cursor;

        Inset::new(
            &mut List::new(self.layout.iter().map(|line| {
                List::new(line.iter().map(|x| CellWidget(x, &cursor)))
                    .with_direction(Direction::Horizontal)
            })),
            Insets::uniform(10.0),
        )
        .layout(ctx, Constraint::unbound());

        ctx.grab_focus();
        ctx.subscribe(Rect::ZERO, EventType::FOCUS | EventType::KEY_DOWN, false);

        for x in ctx.events() {
            debug!("Editor got event: {:?}", x);
            #[allow(clippy::single_match)]
            match x {
                Event::KeyDown(key) => match key.key_code {
                    KeyCode::ArrowLeft => self.move_cursor(0, -1),
                    KeyCode::ArrowUp => self.move_cursor(-1, 0),
                    KeyCode::ArrowDown => self.move_cursor(1, 0),
                    KeyCode::ArrowRight => self.move_cursor(0, 1),
                    _ => {}
                },
                _ => {}
            }
            ctx.invalidate();
        }

        constraint.max
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WithEnumerate<M> {
    pos: CellPosition,
    meta: M,
}

fn enumerate<T, M>(layout: Vec<Vec<SimpleDoc<T, M>>>) -> Vec<Vec<SimpleDoc<T, WithEnumerate<M>>>> {
    layout
        .into_iter()
        .enumerate()
        .map(|(row_id, row)| {
            let mut column = 0;
            row.into_iter()
                .map(|cell| {
                    let cell = cell.map_meta(|meta| WithEnumerate {
                        meta,
                        pos: CellPosition {
                            row: row_id,
                            col: column,
                        },
                    });
                    column += cell.width();
                    cell
                })
                .collect()
        })
        .collect()
}

fn layout_to_2d<T, M>(layout: Vec<SimpleDoc<T, M>>) -> Vec<Vec<SimpleDoc<T, M>>> {
    let mut result = vec![Vec::new()];

    for cell in layout.into_iter() {
        if let SimpleDocKind::Linebreak { .. } = cell.kind {
            result.push(Vec::new());
        }

        let last = result.len() - 1;
        unsafe {
            result.get_unchecked_mut(last).push(cell);
        }
    }

    result
}

pub fn simple_doc_to_string<M>(sdoc: &[SimpleDoc<EditorCellPayload, M>]) -> String {
    let mut out = String::new();

    for doc in sdoc {
        match &doc.kind {
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