use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::{Point, Size, Vec2};

pub struct Center<'a> {
    child: &'a mut dyn Layout,
    child_offset: Vec2,
}

impl<'a> Center<'a> {
    pub fn new(child: &'a mut impl Layout) -> Self {
        Center {
            child,
            child_offset: Vec2::ZERO,
        }
    }
}

impl<'a> Layout for Center<'a> {
    fn set_constraint(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let child_size = self.child.set_constraint(ctx, constraint.to_loose());
        let max = constraint.max;
        let my_size = Size {
            width: if max.width.is_finite() {
                max.width
            } else {
                child_size.width
            },
            height: if max.height.is_finite() {
                max.height
            } else {
                child_size.height
            },
        };
        self.child_offset = (my_size.to_vec2() - child_size.to_vec2()) / 2.0;

        my_size
    }

    fn set_origin(&mut self, origin: Point) {
        self.child.set_origin(origin + self.child_offset);
    }
}
