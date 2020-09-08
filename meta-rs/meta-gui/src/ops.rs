use druid_shell::kurbo::{Point, Rect, RoundedRect, Shape};
use druid_shell::piet::{Color, Piet, RenderContext, Text};

pub(crate) struct Ops<'a> {
    ops: Vec<Op<'a>>,
}

pub(crate) enum Op<'a> {
    SetBrush(<Piet<'a> as RenderContext>::Brush),
    DrawText {
        layout: <<Piet<'a> as RenderContext>::Text as Text>::TextLayout,
        pos: Point,
    },
    Clear(Color),
    Fill(ShapeBox),
    BlurredRect {
        rect: Rect,
        blur_radius: f64,
    },
}

/// A wrapper around common Shapes.
pub(crate) enum ShapeBox {
    RoundedRect(RoundedRect),
}

impl<'a> Ops<'a> {
    pub fn new() -> Self {
        Ops { ops: Vec::new() }
    }

    pub fn push(&mut self, op: Op<'a>) {
        self.ops.push(op);
    }

    pub fn execute(&self, piet: &mut Piet<'a>) {
        let mut current_brush = piet.solid_brush(Color::BLACK);

        for op in self.ops.iter() {
            match op {
                Op::SetBrush(brush) => {
                    current_brush = brush.clone();
                }
                Op::DrawText { layout, pos } => {
                    piet.draw_text(&layout, *pos, &current_brush);
                }
                Op::Clear(color) => piet.clear(color.clone()),
                Op::Fill(shape) => match shape {
                    ShapeBox::RoundedRect(x) => piet.fill(&x, &current_brush),
                },
                Op::BlurredRect { rect, blur_radius } => {
                    piet.blurred_rect(*rect, *blur_radius, &current_brush)
                }
            }
        }
    }
}

impl<'a> Default for Ops<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeBox {
    pub fn from_shape(shape: impl Shape) -> Self {
        if let Some(rounded_rect) = shape.as_rounded_rect() {
            return ShapeBox::RoundedRect(rounded_rect);
        }

        todo!("rest of shapes");
    }
}
