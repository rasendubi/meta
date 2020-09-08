use druid_shell::kurbo::{Point, Size};

use crate::gui::GuiContext;

/// Constraint represent maximum and minimum limits on the widget size.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Constraint {
    pub min: Size,
    pub max: Size,
}

impl Constraint {
    pub fn new(min: Size, max: Size) -> Self {
        Constraint { min, max }
    }

    pub fn loose(max: Size) -> Self {
        Constraint::new(Size::ZERO, max)
    }

    pub fn to_loose(self) -> Self {
        Constraint::loose(self.max)
    }

    pub fn tight(size: Size) -> Self {
        Constraint::new(size, size)
    }

    /// Whether `size` satisfies the constraint.
    pub fn satisfied(&self, size: Size) -> bool {
        self.min.width <= size.width
            && self.max.width >= size.width
            && self.min.height <= size.height
            && self.max.height >= size.height
    }
}

/// Layout trait represent widgets that can be layed out.
pub trait Layout {
    /// Sets the constraints for the widget.
    ///
    /// This function must return the Size widget will occupy (within the provided constraints).
    fn set_constraint(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size;

    /// Set the origin (top left corner) where the widget should be drawn.
    fn set_origin(&mut self, origin: Point);
}
