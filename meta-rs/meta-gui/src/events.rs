use std::collections::HashMap;

use bitflags::bitflags;
use druid_shell::kurbo::{Affine, Rect};
use druid_shell::{KeyEvent, MouseEvent};
use log::trace;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct WidgetId(pub(crate) u64);

bitflags! {
    pub struct EventType: u32 {
        const MOUSE_MOVE = 1 << 0;
        const MOUSE_DOWN = 1 << 1;
        const MOUSE_UP = 1 << 2;
        const MOUSE_LEAVE = 1 << 3;
        const MOUSE_WHEEL = 1 << 4;
        const WIDGET_ENTER = 1 << 5;
        const WIDGET_LEAVE = 1 << 6;
        const KEY_DOWN = 1 << 7;
        const KEY_UP = 1 << 8;
        const FOCUS = 1 << 9;
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseLeave,
    MouseWheel(MouseEvent),
    WidgetEnter,
    WidgetLeave,
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),
    Focus(bool),
}

impl Event {
    fn event_type(&self) -> EventType {
        match self {
            Event::MouseMove(..) => EventType::MOUSE_MOVE,
            Event::MouseDown(..) => EventType::MOUSE_DOWN,
            Event::MouseUp(..) => EventType::MOUSE_UP,
            Event::MouseLeave => EventType::MOUSE_LEAVE,
            Event::MouseWheel(..) => EventType::MOUSE_WHEEL,
            Event::WidgetEnter => EventType::WIDGET_ENTER,
            Event::WidgetLeave => EventType::WIDGET_LEAVE,
            Event::KeyDown(..) => EventType::KEY_DOWN,
            Event::KeyUp(..) => EventType::KEY_UP,
            Event::Focus(..) => EventType::FOCUS,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Subscription {
    pub widget_id: WidgetId,
    pub rect: Rect,
    /// Filter for events. Only the specified events will be delivered to the widget.
    pub events: EventType,
    /// Whether to grab all events. Should be used sparingly.
    ///
    /// The good example usage is to implement drag-and-drop.
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
    focused_widget: Option<WidgetId>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            grab: Vec::new(),
            subscriptions: Vec::new(),
            widget_events: HashMap::new(),
            last_mouse: None,
            focused_widget: None,
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

    pub fn handle_grab_focus_requests(&mut self, requests: Vec<WidgetId>) -> bool {
        for req in requests {
            // the widget has requested focus but it already has one
            if self.focused_widget == Some(req) {
                return false;
            }

            if let Some(widget_id) = self.find_focus_subscription(req).map(|x| x.widget_id) {
                self.focus_widget(widget_id);
                return true;
            }
        }

        false
    }

    fn find_subscription<F: Fn(&Subscription) -> bool>(&self, f: F) -> Option<&Subscription> {
        self.subscriptions.iter().find(|x| f(x))
    }

    fn focus_widget(&mut self, widget_id: WidgetId) {
        if let Some(prev) = self.focused_widget.replace(widget_id) {
            self.widget_events
                .entry(prev)
                .or_insert_with(Vec::new)
                .push(Event::Focus(false));
        }
        self.widget_events
            .entry(widget_id)
            .or_insert_with(Vec::new)
            .push(Event::Focus(true));
    }

    fn find_focus_subscription(&self, req: WidgetId) -> Option<&Subscription> {
        self.find_subscription(|sub| sub.widget_id == req && sub.events.contains(EventType::FOCUS))
    }

    /// Dispatch event to the event queue of the subscribed widget.
    ///
    /// Returns `true` if event was delivered to any widget, `false` otherwise.
    pub fn dispatch(&mut self, event: Event) -> bool {
        let mut dispatched = if let Some(sub) = self.find_subscribed_widget(&event) {
            if sub.events.contains(event.event_type()) {
                let widget_id = sub.widget_id;
                self.widget_events
                    .entry(widget_id)
                    .or_insert_with(Vec::new)
                    .push(event.clone());
                true
            } else {
                false
            }
        } else {
            false
        };

        dispatched |= self.fire_synthetic_events(&event);

        if dispatched {
            trace!("dispatching: {:?}", event);
        } else {
            trace!("not dispatched: {:?}", event);
        }

        dispatched
    }

    fn fire_synthetic_events(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseMove(mouse_event)
            | Event::MouseDown(mouse_event)
            | Event::MouseUp(mouse_event)
            | Event::MouseWheel(mouse_event) => {
                self.dispatch_widget_enter_leave(Some(mouse_event.clone()))
            }
            Event::MouseLeave => self.dispatch_widget_enter_leave(None),
            Event::WidgetEnter | Event::WidgetLeave => {
                panic!(
                    "fire_synthetic_events called with synthetic event {:?}",
                    event
                );
            }
            _ => false,
        }
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

    fn find_subscribed_widget(&self, event: &Event) -> Option<&Subscription> {
        let event_type = event.event_type();

        if let x @ Some(..) = self.grab.iter().find(|x| x.events.contains(event_type)) {
            return x;
        }

        match event {
            Event::MouseMove(mouse)
            | Event::MouseDown(mouse)
            | Event::MouseUp(mouse)
            | Event::MouseWheel(mouse) => {
                if let x @ Some(..) = self.find_subscription(|sub| {
                    sub.events.contains(event_type) && sub.rect.contains(mouse.pos)
                }) {
                    return x;
                }
            }
            Event::WidgetEnter | Event::WidgetLeave | Event::Focus(..) => {
                panic!(
                    "find_subscribed_widget called with synthetic event: {:?}",
                    event
                );
            }
            Event::MouseLeave => {}
            Event::KeyDown(..) | Event::KeyUp(..) => {
                if let Some(focused_widget) = self.focused_widget {
                    if let x @ Some(..) = self.find_subscription(|x| x.widget_id == focused_widget)
                    {
                        return x;
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
