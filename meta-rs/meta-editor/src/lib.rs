pub mod core_layout;
pub mod layout;

use log::debug;

use druid_shell::kurbo::{Insets, Rect, Size};
use druid_shell::piet::{Color, TextLayout};
use druid_shell::Application;
use meta_gui::{Constraint, Direction, EventType, Gui, GuiContext, Inset, Layout, List, Text};

use crate::core_layout::core_layout_entities;
use crate::layout::EditorCellPayload;
use meta_core::MetaCore;
use meta_pretty::{SimpleDoc, SimpleDocKind, WithPath};
use meta_store::MetaStore;

pub fn main(store: MetaStore) {
    let app = Application::new().unwrap();
    let mut editor = Editor::new(store);
    Gui::run(app.clone(), move |ctx| editor.run(ctx));
    app.run(None);
}

enum CursorPosition<M> {
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

type LayoutMeta = WithEnumerate<WithPath<()>>;

struct Editor {
    layout: Vec<Vec<SimpleDoc<EditorCellPayload, LayoutMeta>>>,
    cursor: Option<CursorPosition<LayoutMeta>>,
}

impl Editor {
    pub fn new(store: MetaStore) -> Self {
        let core = MetaCore::new(&store);
        let rich_doc = core_layout_entities(&core).with_path();
        let sdoc = meta_pretty::layout(&rich_doc, 80);
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

    pub fn run(&mut self, ctx: &mut GuiContext) {
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
    }
}

struct CellWidget<'a, M>(
    &'a SimpleDoc<EditorCellPayload, M>,
    &'a Option<CursorPosition<M>>,
);

impl<'a, M> Layout for CellWidget<'a, M>
where
    M: PartialEq,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let string = match &self.0.kind {
            SimpleDocKind::Cell(cell) => cell.payload.text.as_ref().to_string(),
            SimpleDocKind::Linebreak { indent_width } => {
                let mut s = String::with_capacity(*indent_width);
                for _ in 0..*indent_width {
                    s.push(' ');
                }
                s
            }
        };
        let mut text = Text::new(&string).with_font("Input");
        let size = text.layout(ctx, constraint);

        match &self.1 {
            Some(CursorPosition::Inside { cell, offset }) if cell.meta == self.0.meta => {
                let text_layout = text.text_layout(ctx).unwrap();
                let x = text_layout.hit_test_text_position(*offset).unwrap().point.x;
                let brush = ctx.solid_brush(Color::BLACK);
                ctx.fill(Rect::new(x - 0.5, 0.0, x + 0.5, size.height), &brush);
            }
            Some(CursorPosition::Between(_after, before)) if before.meta == self.0.meta => {
                let brush = ctx.solid_brush(Color::BLACK);
                ctx.fill(Rect::new(-0.5, 0.0, 0.5, size.height), &brush);
            }
            _ => {}
        }

        size
    }
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

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct CellPosition {
    row: usize,
    /// Index within the row.
    index: usize,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct WithEnumerate<M> {
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
