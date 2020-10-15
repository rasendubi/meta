use druid_shell::{
    kurbo::{Rect, Size},
    piet::{Color, TextLayout},
};
use unicode_segmentation::UnicodeSegmentation;

use meta_gui::{Constraint, GuiContext, Layout, Text};
use meta_pretty::SimpleDocKind;

use crate::editor::CursorPosition;
use crate::layout::{CellClass, SDoc};

pub(crate) struct CellWidget<'a>(pub &'a SDoc, pub &'a Option<CursorPosition>);

impl<'a> Layout for CellWidget<'a> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let (string, class) = match self.0.kind() {
            SimpleDocKind::Cell(cell) => (
                cell.payload.text.as_ref().to_string(),
                cell.payload.class.clone(),
            ),
            SimpleDocKind::Linebreak { indent_width } => (
                {
                    let mut s = String::with_capacity(*indent_width);
                    for _ in 0..*indent_width {
                        s.push(' ');
                    }
                    s
                },
                CellClass::Whitespace,
            ),
        };

        let text_color = match class {
            CellClass::Editable(..) => Color::rgb8(0x00, 0x30, 0xa6),
            CellClass::Reference(..) => Color::rgb8(0x8f, 0x00, 0x75),
            CellClass::Punctuation => Color::rgb8(0x50, 0x50, 0x50),
            _ => Color::BLACK,
        };
        let mut text = Text::new(&string).with_font("Input").with_color(text_color);
        let (text_size, text_ops) = ctx.capture(|ctx| text.layout(ctx, constraint));

        // Empty strings layout as 0x0 size, which makes empty rows collapse. We still want to show
        // empty rows of proper size, so we draw a placeholder whitespace to calculate the height of
        // the line.
        let min_height = Text::new(&" ")
            .with_font("Input")
            .layout(ctx, constraint)
            .height;
        let size = Size::new(text_size.width, text_size.height.max(min_height));

        match &self.1 {
            Some(CursorPosition::Inside { cell, offset }) if cell == self.0 => {
                let b = ctx.solid_brush(Color::rgba8(0, 0, 0, 20));
                ctx.fill(size.to_rect(), &b);

                ctx.replay(text_ops);
                let text_layout = text.text_layout(ctx).unwrap();
                let grapheme_offset = string
                    .grapheme_indices(true)
                    .nth(*offset)
                    .map_or(string.len(), |x| x.0);
                let x = text_layout
                    .hit_test_text_position(grapheme_offset)
                    .unwrap()
                    .point
                    .x;
                Cursor(x, size.height).layout(ctx, constraint);
            }
            Some(CursorPosition::Between(_after, before)) if before == self.0 => {
                let b = ctx.solid_brush(Color::rgba8(0, 0, 0, 20));
                ctx.fill(size.to_rect(), &b);

                ctx.replay(text_ops);
                Cursor(0.0, size.height).layout(ctx, constraint);
            }
            _ => {
                ctx.replay(text_ops);
            }
        }

        size
    }
}

struct Cursor(f64, f64);

impl Layout for Cursor {
    fn layout(&mut self, ctx: &mut GuiContext, _constraint: Constraint) -> Size {
        let Cursor(x, height) = *self;
        let brush = ctx.solid_brush(Color::BLACK);
        ctx.fill(Rect::new(x - 0.5, 0.0, x + 0.5, height), &brush);

        Size::new(x + 0.5, height)
    }
}
