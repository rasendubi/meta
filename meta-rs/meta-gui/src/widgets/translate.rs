use crate::gui::GuiContext;
use crate::layout::{Constraint, Layout};

use druid_shell::kurbo::{Affine, Size, Vec2};

#[derive(Debug)]
pub struct Translate<T> {
    child: T,
    offset: Vec2,
}

impl<T> Translate<T> {
    pub fn new(child: T, offset: Vec2) -> Self {
        Self { child, offset }
    }

    pub fn child(&self) -> &T {
        &self.child
    }

    pub fn child_mut(&mut self) -> &mut T {
        &mut self.child
    }
}

impl<T> Layout for Translate<T>
where
    T: Layout,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let child_size = ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(self.offset));
            self.child.layout(
                ctx,
                Constraint::new(
                    constraint.min - self.offset.to_size(),
                    constraint.max - self.offset.to_size(),
                ),
            )
        });

        child_size + self.offset.to_size()
    }
}
