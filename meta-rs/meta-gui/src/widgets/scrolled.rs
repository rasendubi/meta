use crate::{Constraint, GuiContext, Layout, Scrollable};
use druid_shell::kurbo::{Affine, Size};

/// A widget that scrolls its child according to the scrollable.
///
/// Takes as much space as needed for the child.
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
        let child_size = ctx.with_save(|ctx| {
            ctx.clip(constraint.max.to_rect());
            ctx.transform(Affine::translate(self.scrollable.offset()));
            self.child.layout(ctx, Constraint::UNBOUND)
        });

        self.scrollable
            .layout(ctx, Constraint::tight(constraint.clamp(child_size)))
    }
}
