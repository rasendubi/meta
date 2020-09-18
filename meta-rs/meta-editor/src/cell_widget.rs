use druid_shell::{
    kurbo::{Rect, Size},
    piet::{Color, TextLayout},
};
use meta_gui::{Constraint, GuiContext, Layout, Text};
use meta_pretty::{SimpleDoc, SimpleDocKind};

use crate::editor::CursorPosition;
use crate::layout::EditorCellPayload;

pub(crate) struct CellWidget<'a, M>(
    pub &'a SimpleDoc<EditorCellPayload, M>,
    pub &'a Option<CursorPosition<M>>,
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
        let (size, text_ops) = ctx.capture(|ctx| text.layout(ctx, constraint));

        match &self.1 {
            Some(CursorPosition::Inside { cell, offset }) if cell.meta == self.0.meta => {
                let b = ctx.solid_brush(Color::rgba8(0, 0, 0, 20));
                ctx.fill(size.to_rect(), &b);

                ctx.replay(text_ops);
                let text_layout = text.text_layout(ctx).unwrap();
                let x = text_layout.hit_test_text_position(*offset).unwrap().point.x;
                let brush = ctx.solid_brush(Color::BLACK);
                ctx.fill(Rect::new(x - 0.5, 0.0, x + 0.5, size.height), &brush);
            }
            Some(CursorPosition::Between(_after, before)) if before.meta == self.0.meta => {
                let b = ctx.solid_brush(Color::rgba8(0, 0, 0, 20));
                ctx.fill(size.to_rect(), &b);

                ctx.replay(text_ops);
                let brush = ctx.solid_brush(Color::BLACK);
                ctx.fill(Rect::new(-0.5, 0.0, 0.5, size.height), &brush);
            }
            _ => {
                ctx.replay(text_ops);
            }
        }

        size
    }
}
