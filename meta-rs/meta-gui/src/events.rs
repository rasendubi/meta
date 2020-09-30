use std::collections::HashMap;

use bitflags::bitflags;
use druid_shell::kurbo::{Affine, Rect};
use druid_shell::{KeyEvent, MouseEvent};
use rand::random;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SubscriptionId(random())
    }
}

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
    /// Id of the subscription. Should remain stable between frames, so that events could find their
    /// way to the subscriber.
    pub id: SubscriptionId,
    /// Screen area to receive events from. Only applies to mouse-initiated events.
    pub rect: Rect,
    /// Filter for events. Only the specified events will be delivered to the subscription.
    pub events: EventType,
    /// Whether to grab all events. Should be used sparingly.
    ///
    /// The good example usage is to implement drag-and-drop.
    pub grab: bool,
}

impl Subscription {
    pub fn transform(self, affine: Affine) -> Self {
        Subscription {
            id: self.id,
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
    subscription_events: HashMap<SubscriptionId, Vec<Event>>,
    last_mouse: Option<MouseEvent>,
    focused: Option<SubscriptionId>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            grab: Vec::new(),
            subscriptions: Vec::new(),
            subscription_events: HashMap::new(),
            last_mouse: None,
            focused: None,
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

    pub fn handle_grab_focus_requests(&mut self, requests: Vec<SubscriptionId>) -> bool {
        for id in requests {
            // the widget has requested focus but it already has one
            if self.focused == Some(id) {
                return false;
            }

            if self.find_focus_subscription(id).is_some() {
                self.focus_subscription(id);
                return true;
            }
        }

        false
    }

    fn find_subscription<F: Fn(&Subscription) -> bool>(&self, f: F) -> Option<&Subscription> {
        self.subscriptions.iter().find(|x| f(x))
    }

    fn find_focus_subscription(&self, id: SubscriptionId) -> Option<&Subscription> {
        self.find_subscription(|sub| sub.id == id && sub.events.contains(EventType::FOCUS))
    }

    fn focus_subscription(&mut self, id: SubscriptionId) {
        if let Some(prev) = self.focused.replace(id) {
            self.subscription_events
                .entry(prev)
                .or_insert_with(Vec::new)
                .push(Event::Focus(false));
        }
        self.subscription_events
            .entry(id)
            .or_insert_with(Vec::new)
            .push(Event::Focus(true));
    }

    /// Dispatch event to the event queue of the subscribed widget.
    ///
    /// Returns `true` if event was delivered to any widget, `false` otherwise.
    pub fn dispatch(&mut self, event: Event) -> bool {
        let mut dispatched = if let Some(sub) = self.find_subscribed_widget(&event) {
            if sub.events.contains(event.event_type()) {
                let id = sub.id;
                self.subscription_events
                    .entry(id)
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
                    self.subscription_events
                        .entry(sub.id)
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
                if let Some(focused) = self.focused {
                    if let x @ Some(..) = self.find_subscription(|x| x.id == focused) {
                        return x;
                    }
                }
            }
        }
        None
    }

    /// Takes events for the subscription, removing them from the queue.
    pub fn take_events(&mut self, id: SubscriptionId) -> Vec<Event> {
        self.subscription_events
            .remove(&id)
            .unwrap_or_else(Vec::new)
    }
}
