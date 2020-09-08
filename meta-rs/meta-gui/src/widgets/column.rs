use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Point, Size, Vec2};

pub struct Column<'a> {
    children: Vec<(&'a mut dyn Layout, Size)>,
}

impl<'a> Column<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Column {
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: &'a mut impl Layout) {
        self.children.push((child, Size::ZERO));
    }
}

impl<'a> Layout for Column<'a> {
    fn set_constraint(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let mut size_left = constraint.max;
        let mut my_size = Size::ZERO;
        for (child, child_size) in self.children.iter_mut() {
            let x = child.set_constraint(ctx, Constraint::loose(size_left));
            *child_size = x;

            size_left.height -= x.height;

            my_size.height += x.height;
            my_size.width = my_size.width.max(x.width);
        }

        my_size
    }

    fn set_origin(&mut self, origin: Point) {
        let mut y_offset = 0.0;
        for (child, size) in self.children.iter_mut() {
            child.set_origin(origin + Vec2::new(0.0, y_offset));
            y_offset += size.height;
        }
    }
}
