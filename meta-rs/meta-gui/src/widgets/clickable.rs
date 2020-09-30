use crate::events::{Event, EventType};
use crate::gui::GuiContext;
use crate::{
    layout::{Constraint, Layout},
    SubscriptionId,
};

use druid_shell::kurbo::Size;

pub struct Click();

/// Stateful widget that implements clickable behavior.
///
/// Takes as little space as possible.
pub struct Clickable {
    id: SubscriptionId,
    pressed: bool,
    hovered: bool,
    clicks: Vec<Click>,
}

impl Clickable {
    pub fn new(id: SubscriptionId) -> Self {
        Clickable {
            id,
            pressed: false,
            hovered: false,
            clicks: Vec::new(),
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Return all clicks that happened after the previous call to `clicks()`.
    pub fn clicks(&mut self) -> Vec<Click> {
        let mut result = Vec::new();
        std::mem::swap(&mut result, &mut self.clicks);
        result
    }
}

impl Layout for Clickable {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let size = constraint.min;
        let rect = size.to_rect();

        for event in ctx.events(self.id) {
            match event {
                Event::MouseDown(..) => {
                    self.pressed = true;
                }
                Event::MouseUp(..) => {
                    if self.pressed && self.hovered {
                        self.clicks.push(Click());
                    }
                    self.pressed = false;
                }
                Event::WidgetEnter => {
                    self.hovered = true;
                }
                Event::WidgetLeave => {
                    self.hovered = false;
                }
                _ => {}
            }
        }

        ctx.subscribe(
            self.id,
            rect,
            EventType::MOUSE_DOWN
                | EventType::MOUSE_UP
                | EventType::WIDGET_ENTER
                | EventType::WIDGET_LEAVE,
            self.pressed,
        );

        size
    }
}
