use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Affine, Size, Vec2};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Direction {
    Horizontal,
    Vertical,
}

pub struct List<I> {
    iter: I,
    direction: Direction,
}

impl<I> List<I> {
    pub fn new(iter: I) -> Self {
        List {
            iter,
            direction: Direction::Vertical,
        }
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}

impl<I: Iterator<Item = Item>, Item: Layout> List<I> {
    fn layout_horizontal(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let mut size_left = constraint.max;
        let mut my_size = Size::ZERO;
        let min_child_size = Size::new(0.0, constraint.min.height);
        for mut child in self.iter.by_ref() {
            let x = ctx.with_save(|ctx| {
                ctx.transform(Affine::translate(Vec2::new(my_size.width, 0.0)));
                child.layout(ctx, Constraint::new(min_child_size, size_left))
            });

            size_left.width -= x.width;

            my_size.width += x.width;
            my_size.height = my_size.height.max(x.height);
        }

        my_size
    }

    fn layout_vertical(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let mut size_left = constraint.max;
        let mut my_size = Size::ZERO;
        let min_child_size = Size::new(constraint.min.width, 0.0);
        for mut child in self.iter.by_ref() {
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

impl<I: Iterator<Item = Item>, Item: Layout> Layout for List<I> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        match self.direction {
            Direction::Horizontal => self.layout_horizontal(ctx, constraint),
            Direction::Vertical => self.layout_vertical(ctx, constraint),
        }
    }
}
