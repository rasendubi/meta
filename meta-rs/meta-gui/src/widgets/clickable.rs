use crate::events::{Event, EventType};
use crate::gui::GuiContext;
use crate::layout::{Constraint, Layout};

use druid_shell::kurbo::Size;

pub struct Click();

pub struct Clickable {
    pressed: bool,
    clicks: Vec<Click>,
}

impl Clickable {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Clickable {
            pressed: false,
            clicks: Vec::new(),
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Return all clicks that happened after the last previous call to `clicks()`.
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

        for event in ctx.events() {
            match event {
                Event::MouseDown(..) => {
                    self.pressed = true;
                }
                Event::MouseUp(..) => {
                    if self.pressed {
                        self.pressed = false;
                        self.clicks.push(Click());
                    }
                }
                _ => {}
            }
        }

        ctx.subscribe(
            rect,
            EventType::MOUSE_DOWN | EventType::MOUSE_UP,
            self.pressed,
        );

        size
    }
}
