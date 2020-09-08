use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Point, Rect, Size};
use druid_shell::piet::{Color, TextLayout};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextPosition {
    TopLeft(Point),
    Baseline(Point),
}

#[derive(Debug)]
pub struct Text<'a> {
    text: &'a str,
    position: TextPosition,
    font_name: &'a str,
    font_size: f64,
    color: Color,
    width: f64,
}

impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Self {
        Text {
            text,
            position: TextPosition::TopLeft(Point::ZERO),
            font_name: "Roboto",
            font_size: 8.5,
            color: Color::BLACK,
            width: f64::INFINITY,
        }
    }

    pub fn with_position(mut self, position: TextPosition) -> Self {
        self.position = position;
        self
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

    pub fn draw(self, ctx: &mut GuiContext) {
        let font = ctx
            .new_font_by_name(self.font_name, self.font_size)
            .unwrap();

        let text_layout = ctx.new_text_layout(&font, self.text, None).unwrap();

        let last_line = text_layout
            .line_metric(text_layout.line_count() - 1)
            .unwrap();
        let text_size = Size::new(text_layout.width(), last_line.cumulative_height);
        let text_baseline = last_line.baseline;

        let point = match self.position {
            TextPosition::Baseline(point) => point,
            TextPosition::TopLeft(point) => {
                let text_rect = Rect::from_origin_size(point, text_size);
                Point::new(text_rect.x0, text_rect.y0 + text_baseline)
            }
        };

        let text_brush = ctx.solid_brush(self.color);
        ctx.draw_text(&text_layout, point, &text_brush);
    }
}

impl Layout for Text<'_> {
    fn set_constraint(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        self.width = constraint.max.width;

        let font = ctx
            .new_font_by_name(self.font_name, self.font_size)
            .unwrap();

        let text_layout = ctx
            .new_text_layout(&font, self.text, Some(self.width))
            .unwrap();

        let last_line = text_layout
            .line_metric(text_layout.line_count() - 1)
            .unwrap();

        Size::new(text_layout.width(), last_line.cumulative_height)
    }

    fn set_origin(&mut self, origin: Point) {
        self.position = TextPosition::TopLeft(origin);
    }
}
