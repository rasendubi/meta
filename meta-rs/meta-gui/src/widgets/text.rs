use crate::gui::GuiContext;

use druid_shell::kurbo::{Point, Rect, Size};
use druid_shell::piet::{
    Color, FontBuilder, RenderContext, Text as _, TextLayout, TextLayoutBuilder,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextPosition {
    TopLeft(Point),
    Baseline(Point),
    Center(Point),
    // TODO: CenterVertical, CenterHorizontal
}

#[derive(Debug)]
pub struct Text<'a> {
    text: &'a str,
    position: TextPosition,
    font: (&'a str, f64),
    color: Color,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str, position: TextPosition) -> Self {
        Text {
            text,
            position,
            font: ("Roboto Medium", 7.0),
            color: Color::BLACK,
        }
    }

    pub fn with_font(mut self, name: &'a str, size: f64) -> Self {
        self.font = (name, size);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn draw(self, ctx: &mut GuiContext) {
        let text = ctx.piet.text();

        let (font_name, font_size) = self.font;
        let font = text.new_font_by_name(font_name, font_size).build().unwrap();

        let text_layout = text
            .new_text_layout(&font, self.text, None)
            .build()
            .unwrap();

        let last_line = text_layout
            .line_metric(text_layout.line_count() - 1)
            .unwrap();
        let text_size = Size::new(text_layout.width(), last_line.cumulative_height);
        let text_baseline = last_line.baseline;

        let point = match self.position {
            TextPosition::Baseline(point) => point,
            TextPosition::Center(point) => {
                let text_rect = Rect::from_center_size(point, text_size);
                Point::new(text_rect.x0, text_rect.y0 + text_baseline)
            }
            TextPosition::TopLeft(point) => {
                let text_rect = Rect::from_origin_size(point, text_size);
                Point::new(text_rect.x0, text_rect.y0 + text_baseline)
            }
        };

        let text_brush = ctx.piet.solid_brush(self.color);
        ctx.piet.draw_text(&text_layout, point, &text_brush);
    }
}
