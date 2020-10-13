use druid_shell::kurbo::Size;

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

    pub const UNBOUND: Constraint = Constraint {
        min: Size::ZERO,
        max: Size::new(f64::INFINITY, f64::INFINITY),
    };

    pub fn loose(max: Size) -> Self {
        Constraint::new(Size::ZERO, max)
    }

    pub fn tight(size: Size) -> Self {
        Constraint::new(size, size)
    }

    pub fn to_loose(self) -> Self {
        Constraint::loose(self.max)
    }

    /// Whether `size` satisfies the constraint.
    pub fn satisfied(&self, size: Size) -> bool {
        self.min.width <= size.width
            && self.max.width >= size.width
            && self.min.height <= size.height
            && self.max.height >= size.height
    }

    /// Adjust `size` to satisfy the constraint.
    ///
    /// If length of any side is shorter than minimum allowed constraint, the minimum is
    /// returned. If the length is longer than maximum, the maximum is returned.
    pub fn clamp(&self, size: Size) -> Size {
        size.clamp(self.min, self.max)
    }
}

/// Layout trait represent widgets that can be layed out.
pub trait Layout {
    /// Sets the constraints for the widget.
    ///
    /// This function must return the Size widget will occupy (within the provided constraints).
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size;
}

impl<'a, T> Layout for &'a mut T
where
    T: Layout,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        (*self).layout(ctx, constraint)
    }
}

impl<'a> Layout for &'a mut dyn Layout {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        (*self).layout(ctx, constraint)
    }
}

impl<T> Layout for Option<T>
where
    T: Layout,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        match self {
            Some(x) => x.layout(ctx, constraint),
            None => Size::ZERO,
        }
    }
}
