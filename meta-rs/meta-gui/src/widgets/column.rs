use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Affine, Size, Vec2};

pub struct Column<T> {
    children: Vec<T>,
}

impl<T> Column<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Column {
            children: Vec::new(),
        }
    }

    pub fn with_child(mut self, child: T) -> Self {
        self.children.push(child);
        self
    }
}

impl<T> Layout for Column<T>
where
    T: Layout,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let mut size_left = constraint.max;
        let mut my_size = Size::ZERO;
        let min_child_size = Size::new(constraint.min.width, 0.0);
        for child in self.children.iter_mut() {
            let x = ctx.with_save(|ctx| {
                ctx.transform(Affine::translate(Vec2::new(0.0, my_size.height)));
                child.layout(ctx, Constraint::new(min_child_size, size_left))
            });

            size_left.height -= x.height;

            my_size.height += x.height;
            my_size.width = my_size.width.max(x.width);
        }

        my_size
    }
}
