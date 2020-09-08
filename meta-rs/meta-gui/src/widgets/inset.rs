use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Affine, Insets, Size, Vec2};

pub struct Inset<'a> {
    child: &'a mut dyn Layout,
    insets: Insets,
}

impl<'a> Inset<'a> {
    pub fn new(child: &'a mut dyn Layout, insets: Insets) -> Self {
        Inset { child, insets }
    }
}

impl<'a> Layout for Inset<'a> {
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
