use crate::{Constraint, GuiContext, Layout};
use druid_shell::{
    kurbo::{Rect, Size},
    piet::Color,
};

/// Vertical scrollbar.
///
/// Takes as little space as allowed.
pub struct Scrollbar {
    offset: f64,
    height: f64,
}

impl Scrollbar {
    /// `offset` from the top, in percents.
    /// `height` of the scrollbar in percents.
    pub fn new(offset: f64, height: f64) -> Self {
        Scrollbar { offset, height }
    }
}

impl Layout for Scrollbar {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let size = constraint.min;

        let scrollbar_height = self.height * size.height;
        let scrollbar_offset = self.offset * size.height;
        let background = ctx.solid_brush(Color::rgb8(0xe0, 0xe0, 0xe0));
        let foreground = ctx.solid_brush(Color::rgb8(0x42, 0x42, 0x42));

        ctx.fill(size.to_rect(), &background);
        ctx.fill(
            Rect::new(
                1.0,
                scrollbar_offset,
                size.width - 1.0,
                scrollbar_offset + scrollbar_height,
            )
            .to_rounded_rect(1.0),
            &foreground,
        );

        size
    }
}
