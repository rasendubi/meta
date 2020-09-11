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
        const MOUSE_LEAVE = 1 << 3;
        const WIDGET_ENTER = 1 << 4;
        const WIDGET_LEAVE = 1 << 5;
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseLeave,
    WidgetEnter,
    WidgetLeave,
}

impl Event {
    fn event_type(&self) -> EventType {
        match self {
            Event::MouseMove(..) => EventType::MOUSE_MOVE,
            Event::MouseDown(..) => EventType::MOUSE_DOWN,
            Event::MouseUp(..) => EventType::MOUSE_UP,
            Event::MouseLeave => EventType::MOUSE_LEAVE,
            Event::WidgetEnter => EventType::WIDGET_ENTER,
            Event::WidgetLeave => EventType::WIDGET_LEAVE,
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
pub(crate) struct EventQueue {
    grab: Vec<Subscription>,
    subscriptions: Vec<Subscription>,
    widget_events: HashMap<WidgetId, Vec<Event>>,
    last_mouse: Option<MouseEvent>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            grab: Vec::new(),
            subscriptions: Vec::new(),
            widget_events: HashMap::new(),
            last_mouse: None,
        }
    }

    pub fn subscribe(&mut self, sub: Subscription) {
        if sub.grab {
            self.grab.push(sub);
        }
        self.subscriptions.push(sub);
    }

    pub fn clear_subscriptions(&mut self) {
        self.grab.clear();
        self.subscriptions.clear();
    }

    pub fn replace_subscriptions(&mut self, subscriptions: Vec<Subscription>) {
        self.clear_subscriptions();
        for sub in subscriptions {
            self.subscribe(sub);
        }
    }

    /// Dispatch event to the event queue of the subscribed widget.
    ///
    /// Returns `true` if event was delivered to any widget, `false` otherwise.
    pub fn dispatch(&mut self, event: Event) -> bool {
        let mut dispatched = if let Some(widget_id) = self.find_subscribed_widget(&event) {
            self.widget_events
                .entry(widget_id)
                .or_insert_with(Vec::new)
                .push(event.clone());
            true
        } else {
            false
        };

        match event {
            Event::MouseMove(mouse_event)
            | Event::MouseDown(mouse_event)
            | Event::MouseUp(mouse_event) => {
                dispatched |= self.dispatch_widget_enter_leave(Some(mouse_event));
            }
            Event::MouseLeave => {
                dispatched |= self.dispatch_widget_enter_leave(None);
            }
            Event::WidgetEnter | Event::WidgetLeave => {
                panic!("dispatch called with WidgetEnter | WidgetLeave");
            }
        }

        dispatched
    }

    fn dispatch_widget_enter_leave(&mut self, new: Option<MouseEvent>) -> bool {
        let prev = self.last_mouse.take();
        self.last_mouse = new;

        let mut dispatched = false;
        for sub in self.subscriptions.iter() {
            let last_in = prev
                .as_ref()
                .map_or(false, |mouse| sub.rect.contains(mouse.pos));
            let new_in = self
                .last_mouse
                .as_ref()
                .map_or(false, |mouse| sub.rect.contains(mouse.pos));

            if last_in != new_in {
                let event = if new_in {
                    Event::WidgetEnter
                } else {
                    Event::WidgetLeave
                };
                if sub.events.contains(event.event_type()) {
                    self.widget_events
                        .entry(sub.widget_id)
                        .or_insert_with(Vec::new)
                        .push(event);
                    dispatched = true;
                }
            }
        }

        dispatched
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
            Event::MouseLeave => {}
            Event::WidgetEnter | Event::WidgetLeave => {
                panic!("find_subscribed_widget called with Enter | Leave");
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
