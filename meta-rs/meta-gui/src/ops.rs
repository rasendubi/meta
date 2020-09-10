use druid_shell::kurbo::{Affine, Point, Rect, RoundedRect, Shape};
use druid_shell::piet::{Color, Piet, RenderContext, Text};

use crate::subscriptions::{Subscription, Subscriptions};

pub struct Ops<'a> {
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
    Transform(Affine),
    Save,
    Restore,
    Subscribe(Subscription),
}

/// A wrapper around common Shapes.
pub(crate) enum ShapeBox {
    RoundedRect(RoundedRect),
}

impl<'a> Ops<'a> {
    pub(crate) fn new() -> Self {
        Ops { ops: Vec::new() }
    }

    pub(crate) fn push(&mut self, op: Op<'a>) {
        self.ops.push(op);
    }

    pub(crate) fn push_all(&mut self, mut ops: Ops<'a>) {
        self.ops.append(&mut ops.ops);
    }

    pub(crate) fn execute(&self, piet: &mut Piet<'a>) -> Subscriptions {
        let mut subscriptions = Subscriptions::new();

        let mut state_stack = Vec::new();

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
                Op::Transform(transform) => {
                    piet.transform(*transform);
                }
                Op::Save => {
                    state_stack.push((current_brush.clone(),));
                    piet.save().unwrap();
                }
                Op::Restore => {
                    let (brush,) = state_stack.pop().unwrap();
                    current_brush = brush;
                    piet.restore().unwrap();
                }
                Op::Subscribe(sub) => {
                    subscriptions.subscribe(sub.transform(piet.current_transform()));
                }
            }
        }

        subscriptions
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
