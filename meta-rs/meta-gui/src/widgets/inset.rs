use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Affine, Insets, Size, Vec2};

pub struct Inset<T> {
    child: T,
    insets: Insets,
}

impl<T> Inset<T> {
    pub fn new(child: T, insets: Insets) -> Self {
        Inset { child, insets }
    }
}

impl<T> Layout for Inset<T>
where
    T: Layout,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let child_size = ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(Vec2::new(self.insets.x0, self.insets.y0)));
            self.child.layout(
                ctx,
                Constraint::new(
                    constraint.min - self.insets.size(),
                    constraint.max - self.insets.size(),
                ),
            )
        });

        child_size + self.insets.size()
    }
}
