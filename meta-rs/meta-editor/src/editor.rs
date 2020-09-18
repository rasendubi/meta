use log::Level::Trace;
use log::{debug, log_enabled, trace};

use druid_shell::kurbo::{Insets, Rect, Size};
use druid_shell::piet::Color;
use meta_gui::{Constraint, Direction, EventType, GuiContext, Inset, Layout, List};

use crate::cell_widget::CellWidget;
use crate::core_layout::core_layout_entities;
use crate::layout::EditorCellPayload;
use meta_core::MetaCore;
use meta_pretty::{SimpleDoc, SimpleDocKind, WithPath};
use meta_store::MetaStore;

pub type LayoutMeta = WithEnumerate<WithPath<()>>;

pub struct Editor {
    layout: Vec<Vec<SimpleDoc<EditorCellPayload, LayoutMeta>>>,
    cursor: Option<CursorPosition<LayoutMeta>>,
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

        let cursor = layout
            .first()
            .and_then(|x| x.first())
            .map(|cell| CursorPosition::Inside {
                cell: cell.clone(),
                offset: 0,
            });

        Editor { layout, cursor }
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
        ctx.subscribe(Rect::ZERO, EventType::FOCUS | EventType::KEY_UP, false);

        for x in ctx.events() {
            debug!("Editor got event: {:?}", x);
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
            row.into_iter()
                .enumerate()
                .map(|(index, cell)| {
                    cell.map_meta(|meta| WithEnumerate {
                        meta,
                        pos: CellPosition { row: row_id, index },
                    })
                })
                .collect()
        })
        .collect()
}

pub enum CursorPosition<M> {
    Inside {
        cell: SimpleDoc<EditorCellPayload, M>,
        offset: usize,
    },
    #[allow(dead_code)]
    Between(
        SimpleDoc<EditorCellPayload, M>,
        SimpleDoc<EditorCellPayload, M>,
    ),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellPosition {
    row: usize,
    /// Index within the row.
    index: usize,
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
