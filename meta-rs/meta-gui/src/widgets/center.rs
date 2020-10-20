use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Affine, Size};

#[derive(Debug)]
pub struct Center<T> {
    child: T,
}

impl<T> Center<T> {
    pub fn new(child: T) -> Self {
        Center { child }
    }
}

impl<T> Layout for Center<T>
where
    T: Layout,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let (child_size, ops) = ctx.capture(|ctx| self.child.layout(ctx, constraint.to_loose()));
        let max = constraint.max;
        let my_size = Size {
            width: if max.width.is_finite() {
                max.width
            } else {
                child_size.width
            },
            height: if max.height.is_finite() {
                max.height
            } else {
                child_size.height
            },
        };

        let child_offset = (my_size.to_vec2() - child_size.to_vec2()) / 2.0;

        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(child_offset));
            ctx.replay(ops);
        });

        my_size
    }
}
