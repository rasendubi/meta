use crate::gui::GuiContext;
use crate::layout::{Constraint, Layout};

use druid_shell::kurbo::Size;

/// Stack layout.
///
/// Draws children on top of each other. Has the size of the largest child.
#[derive(Debug)]
pub struct Stack<I> {
    iter: I,
}

impl<I> Stack<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator<Item = Item>, Item: Layout> Layout for Stack<I> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let mut max_size = Size::ZERO;
        for mut child in self.iter.by_ref() {
            let size = child.layout(ctx, constraint);

            max_size.width = max_size.width.max(size.width);
            max_size.height = max_size.height.max(size.height);
        }

        constraint.clamp(max_size)
    }
}
