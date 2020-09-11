use std::collections::HashMap;

use bitflags::bitflags;
use druid_shell::kurbo::{Affine, Rect};
use druid_shell::MouseEvent;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct WidgetId(pub(crate) u64);

bitflags! {
    pub struct EventType: u32 {
        const MOUSE_MOVE = 1 << 0;
        const MOUSE_DOWN = 1 << 1;
        const MOUSE_UP = 1 << 2;
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
}

impl Event {
    fn event_type(&self) -> EventType {
        match self {
            Event::MouseMove(..) => EventType::MOUSE_MOVE,
            Event::MouseDown(..) => EventType::MOUSE_DOWN,
            Event::MouseUp(..) => EventType::MOUSE_UP,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Subscription {
    pub widget_id: WidgetId,
    pub rect: Rect,
    pub events: EventType,
    /// Whether to grab all events. Should be used sparingly.
    pub grab: bool,
}

impl Subscription {
    pub fn transform(self, affine: Affine) -> Self {
        Subscription {
            widget_id: self.widget_id,
            rect: affine.transform_rect_bbox(self.rect),
            events: self.events,
            grab: self.grab,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Subscriptions {
    grab: Vec<Subscription>,
    subscriptions: Vec<Subscription>,
    widget_events: HashMap<WidgetId, Vec<Event>>,
}

impl Subscriptions {
    pub fn new() -> Self {
        Subscriptions {
            grab: Vec::new(),
            subscriptions: Vec::new(),
            widget_events: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, sub: Subscription) {
        if sub.grab {
            self.grab.push(sub);
        }
        self.subscriptions.push(sub);
    }

    /// Dispatch event to the event queue of the subscribed widget.
    ///
    /// Returns `true` if event was delivered to any widget, `false` otherwise.
    pub fn dispatch(&mut self, event: Event) -> bool {
        if let Some(widget_id) = self.find_subscribed_widget(&event) {
            self.widget_events
                .entry(widget_id)
                .or_insert_with(Vec::new)
                .push(event);
            true
        } else {
            false
        }
    }

    fn find_subscribed_widget(&self, event: &Event) -> Option<WidgetId> {
        let event_type = event.event_type();

        for sub in self.grab.iter() {
            if sub.events.contains(event_type) {
                return Some(sub.widget_id);
            }
        }

        match event {
            Event::MouseMove(mouse) | Event::MouseDown(mouse) | Event::MouseUp(mouse) => {
                for sub in self.subscriptions.iter() {
                    if sub.events.contains(event_type) && sub.rect.contains(mouse.pos) {
                        return Some(sub.widget_id);
                    }
                }
            }
        }
        None
    }

    pub fn widget_events(&mut self, widget_id: WidgetId) -> Vec<Event> {
        self.widget_events
            .remove(&widget_id)
            .unwrap_or_else(Vec::new)
    }
}
