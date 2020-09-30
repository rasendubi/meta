use crate::{Constraint, Event, EventType, GuiContext, Layout};
use druid_shell::kurbo::{Size, Vec2};
use log::trace;

/// Scrollable is a stateful widget behavior to allow scrolling other widgets and areas.
///
/// It does not draw the child widget itself but should be used with a companion widget that knows
/// to scroll its child and add decorations, etc.
pub struct Scrollable {
    offset: Vec2,
}

impl Scrollable {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Scrollable { offset: Vec2::ZERO }
    }

    pub fn offset(&self) -> Vec2 {
        self.offset
    }
}

impl Layout for Scrollable {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let size = constraint.max;
        let rect = size.to_rect();

        for event in ctx.events() {
            trace!("got event: {:?}", event);
            if let Event::MouseWheel(mouse) = event {
                let delta = mouse.wheel_delta;
                self.offset -= delta / 3.0;

                trace!("delta: {:?}", delta);

                ctx.invalidate();
            }
        }

        ctx.subscribe(rect, EventType::MOUSE_WHEEL, false);

        size
    }
}
