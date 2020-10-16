use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::{Color, Piet, RenderContext, Text as PietText, TextLayout};

#[derive(Debug)]
pub struct Text<'a, T> {
    text: T,
    font_name: &'a str,
    font_size: f64,
    color: Color,
    width: f64,
}

impl<'a, T> Text<'a, T>
where
    T: AsRef<str>,
{
    pub fn new(text: T) -> Self {
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

        ctx.new_text_layout(&font, self.text.as_ref(), Some(self.width))
    }
}

impl<T> Layout for Text<'_, T>
where
    T: AsRef<str>,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        self.width = constraint.max.width;

        let text_layout = self.text_layout(ctx).unwrap();

        let line_count = text_layout.line_count();
        // TODO: in piet-cairo-0.2.0, line_metric always works for line 0 and returns default line
        // height. Remove this early-return and placeholder workaround in Cell widget.
        if line_count == 0 {
            return Size::ZERO;
        }

        let last_line = text_layout.line_metric(line_count - 1).unwrap();
        let text_size = Size::new(text_layout.width(), last_line.cumulative_height);
        let text_baseline = last_line.baseline;

        // Piet's text drawing is very bad and it can't even properly draw monospaced fonts. Draw
        // each char separately, so it looks normal.
        let font = ctx
            .new_font_by_name(self.font_name, self.font_size)
            .unwrap();
        let text_brush = ctx.solid_brush(self.color.clone());
        let mut x_offset = 0.0;
        for c in self.text.as_ref().chars() {
            let layout = ctx.new_text_layout(&font, &c.to_string(), None).unwrap();
            let point = Point::new(x_offset, text_baseline);
            ctx.draw_text(&layout, point, &text_brush);

            x_offset += layout.width();
        }

        constraint.clamp(text_size)
    }
}
