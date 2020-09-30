use crate::{Constraint, GuiContext, Layout, Scrollable};
use druid_shell::kurbo::{Affine, Size, Vec2};

/// A widget that scrolls its child according to the scrollable.
///
/// Takes at most as much space as needed for the child. Does not allow scrolling past the edges of
/// child.
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
        let size = ctx.with_save(|ctx| {
            ctx.clip(constraint.max.to_rect());
            ctx.transform(Affine::translate(-self.scrollable.offset()));
            self.child.layout(
                ctx,
                Constraint::new(constraint.min, Size::new(f64::INFINITY, f64::INFINITY)),
            )
        });

        let widget_size = constraint.clamp(size);
        self.scrollable.layout(ctx, Constraint::tight(widget_size));

        let offset = self.scrollable.offset();
        if offset.y < 0.0 {
            self.scrollable.set_offset(Vec2::new(offset.x, 0.0));
            ctx.invalidate();
        } else if size.height - offset.y < widget_size.height {
            self.scrollable
                .set_offset(Vec2::new(offset.x, size.height - widget_size.height));
            ctx.invalidate();
        }

        widget_size
    }
}
