use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Affine, Size, Vec2};

pub struct Row<'a> {
    children: Vec<&'a mut dyn Layout>,
}

impl<'a> Row<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Row {
            children: Vec::new(),
        }
    }

    pub fn with_child(mut self, child: &'a mut impl Layout) -> Self {
        self.children.push(child);
        self
    }
}

impl<'a> Layout for Row<'a> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let mut size_left = constraint.max;
        let mut my_size = Size::ZERO;
        for child in self.children.iter_mut() {
            let x = ctx.with_save(|ctx| {
                ctx.transform(Affine::translate(Vec2::new(my_size.width, 0.0)));
                child.layout(ctx, Constraint::loose(size_left))
            });

            size_left.width -= x.width;

            my_size.width += x.width;
            my_size.height = my_size.height.max(x.height);
        }

        my_size
    }
}
