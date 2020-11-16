use druid_shell::kurbo::{Affine, Insets, Rect, Size, Vec2};
use log::trace;

use crate::gui::GuiContext;
use crate::layout::*;
use crate::widgets::{Scrollable, Scrollbar, Translate};

/// `ScrolledList` combines a vertical `List` and `Scrolled` and performs additional optimization:
/// it does not draw/layout children that are out of view. This significantly reduces the amount of
/// work (and time) required to render a frame.
///
/// As `Scrolled`, `ScrolledList` does not allow scrolling past the edges of child.
#[derive(Debug)]
pub struct ScrolledList<'a, I> {
    scrollable: &'a mut Scrollable,
    item_height: f64,
    iter: I,
    insets: Insets,
}

impl<'a, I> ScrolledList<'a, I> {
    pub fn new(scrollable: &'a mut Scrollable, item_height: f64, iter: I) -> Self {
        Self {
            scrollable,
            item_height,
            iter,
            insets: Insets::ZERO,
        }
    }

    pub fn with_insets(mut self, insets: Insets) -> Self {
        self.insets = insets;
        self
    }
}

impl<'a, I: Iterator<Item = Item> + ExactSizeIterator, Item: Layout> Layout
    for ScrolledList<'a, I>
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let (child_count, _) = self.iter.size_hint();
        let item_height = self.item_height;
        let list_height = (child_count as f64) * self.item_height;
        let inner_height = list_height + self.insets.y_value();

        let scrollbar_width = 6.0;

        let size = constraint.clamp(Size::new(
            self.insets.x_value() + scrollbar_width,
            inner_height,
        ));
        self.scrollable.layout(ctx, Constraint::tight(size));

        let scroll_offset = self.scrollable.offset();
        let visible_rect = Rect::from_origin_size(scroll_offset.to_point(), size);
        for (i, mut child) in self.iter.by_ref().enumerate() {
            let child_start = (i as f64) * self.item_height + self.insets.y0;
            let child_end = ((i + 1) as f64) * self.item_height + self.insets.y0;
            let child_offset = Vec2::new(self.insets.x0, child_start);

            if child_end < visible_rect.y0 {
                // the child is not visible
                continue;
            }
            if child_start > visible_rect.y1 {
                // the child is not visible, and none of the following child are
                break;
            }

            ctx.with_save(|ctx| {
                ctx.transform(Affine::translate(-scroll_offset + child_offset));

                child.layout(
                    ctx,
                    Constraint::new(Size::ZERO, Size::new(f64::INFINITY, item_height)),
                );
            });
        }
        Translate::new(
            Scrollbar::new(scroll_offset.y / inner_height, size.height / inner_height),
            Vec2::new(size.width - scrollbar_width, 0.0),
        )
        .layout(ctx, Constraint::tight(size));

        let max_x_offset = f64::INFINITY;
        let max_y_offset = inner_height - size.height;
        let next_offset = Vec2::new(
            scroll_offset.x.max(0.0).min(max_x_offset),
            scroll_offset.y.max(0.0).min(max_y_offset),
        );
        trace!(
            "size: {:?}, list_height: {:?}, scroll_offset: {:?}, next_offset: {:?}",
            size,
            list_height,
            scroll_offset,
            next_offset
        );
        if next_offset != scroll_offset {
            self.scrollable.set_offset(next_offset);
            trace!("invalidate!");
            ctx.invalidate();
        }

        size
    }
}
