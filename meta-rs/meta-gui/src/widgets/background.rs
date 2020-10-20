use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::{kurbo::Size, piet::Color};

#[derive(Debug)]
pub struct Background<T> {
    color: Color,
    child: T,
}

impl<T> Background<T> {
    pub fn new(child: T) -> Self {
        Self {
            color: Color::WHITE,
            child,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl<T> Layout for Background<T>
where
    T: Layout,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let (child_size, ops) = ctx.capture(|ctx| self.child.layout(ctx, constraint));

        let brush = ctx.solid_brush(self.color.clone());
        ctx.fill(child_size.to_rect(), &brush);

        ctx.replay(ops);
        child_size
    }
}
