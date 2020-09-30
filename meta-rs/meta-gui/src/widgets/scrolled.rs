use crate::{Constraint, GuiContext, Layout, Scrollable};
use druid_shell::kurbo::{Affine, Size};

pub struct Scrolled<'a> {
    scrollable: &'a mut Scrollable,
    child: &'a mut dyn Layout,
}

impl<'a> Scrolled<'a> {
    pub fn new(scrollable: &'a mut Scrollable, child: &'a mut dyn Layout) -> Self {
        Scrolled { scrollable, child }
    }
}

impl<'a> Layout for Scrolled<'a> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(self.scrollable.offset()));
            self.child.layout(ctx, constraint);
        });

        self.scrollable.layout(ctx, constraint)
    }
}
