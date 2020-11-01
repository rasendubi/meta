use druid_shell::kurbo::{Affine, Size, Vec2};
use log::trace;

use crate::widgets::{Scrollable, Scrollbar};
use crate::{Constraint, GuiContext, Layout};

/// A widget that scrolls its child according to the scrollable.
///
/// Takes at most as much space as needed for the child. Does not allow scrolling past the edges of
/// child.
#[derive(Debug)]
pub struct Scrolled<'a, T> {
    scrollable: &'a mut Scrollable,
    child: T,
}

impl<'a, T> Scrolled<'a, T> {
    pub fn new(scrollable: &'a mut Scrollable, child: T) -> Self {
        Scrolled { scrollable, child }
    }
}

impl<'a, T> Layout for Scrolled<'a, T>
where
    T: Layout,
{
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

        let scrollbar_width = 6.0;
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(Vec2::new(
                widget_size.width - scrollbar_width,
                0.0,
            )));
            Scrollbar::new(offset.y / size.height, widget_size.height / size.height).layout(
                ctx,
                Constraint::tight(Size::new(scrollbar_width, widget_size.height)),
            );
        });

        let max_x_offset = size.width - widget_size.width;
        let max_y_offset = size.height - widget_size.height;
        let next_offset = Vec2::new(
            offset.x.max(0.0).min(max_x_offset),
            offset.y.max(0.0).min(max_y_offset),
        );
        trace!(
            "size: {:?}, widget_size: {:?}, offset: {:?}, next_offset: {:?}",
            size,
            widget_size,
            offset,
            next_offset
        );
        if next_offset != offset {
            self.scrollable.set_offset(next_offset);
            trace!("invalidate!");
            ctx.invalidate();
        }

        widget_size
    }
}
