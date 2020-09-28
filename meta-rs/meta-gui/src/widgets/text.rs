use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::{Color, Piet, RenderContext, Text as PietText, TextLayout};

#[derive(Debug)]
pub struct Text<'a> {
    text: &'a str,
    font_name: &'a str,
    font_size: f64,
    color: Color,
    width: f64,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Self {
        Text {
            text,
            font_name: "Roboto",
            // Piet's text layout is awful. Text width reported by TextLayout is always wrong and
            // the error offset depends on the font size. With this font size the error is bearable
            // (at least for short strings).
            font_size: 10.0,
            color: Color::BLACK,
            width: f64::INFINITY,
        }
    }

    pub fn with_font(mut self, name: &'a str) -> Self {
        self.font_name = name;
        self
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn text_layout(
        &self,
        ctx: &mut GuiContext,
    ) -> Result<<<Piet as RenderContext>::Text as PietText>::TextLayout, druid_shell::piet::Error>
    {
        let font = ctx.new_font_by_name(self.font_name, self.font_size)?;

        ctx.new_text_layout(&font, self.text, Some(self.width))
    }
}

impl Layout for Text<'_> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        self.width = constraint.max.width;

        let text_layout = self.text_layout(ctx).unwrap();

        if text_layout.line_count() == 0 {
            return Size::ZERO;
        }

        let last_line = text_layout
            .line_metric(text_layout.line_count() - 1)
            .unwrap();
        let text_size = Size::new(text_layout.width(), last_line.cumulative_height);
        let text_baseline = last_line.baseline;

        // Piet's text drawing is very bad and it can't even properly draw monospaced fonts. Draw
        // each char separately, so it looks normal.
        let font = ctx
            .new_font_by_name(self.font_name, self.font_size)
            .unwrap();
        let text_brush = ctx.solid_brush(self.color.clone());
        let mut x_offset = 0.0;
        for c in self.text.chars() {
            let layout = ctx.new_text_layout(&font, &c.to_string(), None).unwrap();
            let point = Point::new(x_offset, text_baseline);
            ctx.draw_text(&layout, point, &text_brush);

            x_offset += layout.width();
        }

        text_size
    }
}
