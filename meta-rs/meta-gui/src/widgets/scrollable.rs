use crate::{Constraint, Event, EventType, GuiContext, Layout, SubscriptionId};
use druid_shell::kurbo::{Size, Vec2};

/// Scrollable is a stateful widget behavior to allow scrolling other widgets and areas.
///
/// It does not draw the child widget itself but should be used with a companion widget that knows
/// to scroll its child and add decorations, etc.
///
/// Takes as little space as possible.
#[derive(Debug)]
pub struct Scrollable {
    id: SubscriptionId,
    offset: Vec2,
}

impl Scrollable {
    pub fn new(id: SubscriptionId) -> Self {
        Scrollable {
            id,
            offset: Vec2::ZERO,
        }
    }

    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }
}

impl Layout for Scrollable {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let size = constraint.min;

        for event in ctx.events(self.id) {
            if let Event::MouseWheel(mouse) = event {
                let delta = mouse.wheel_delta;
                self.offset += delta / 3.0;

                ctx.invalidate();
            }
        }

        ctx.subscribe(self.id, size.to_rect(), EventType::MOUSE_WHEEL, false);

        size
    }
}
