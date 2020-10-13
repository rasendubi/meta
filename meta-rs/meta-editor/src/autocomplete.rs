use druid_shell::kurbo::{Insets, Size};
use druid_shell::piet::Color;
use druid_shell::{HotKey, KeyCode, RawMods};
use log::trace;

use meta_gui::{
    Background, Constraint, Event, EventType, GuiContext, Inset, Layout, List, SubscriptionId, Text,
};

pub struct Autocomplete<T> {
    id: SubscriptionId,
    candidates: Vec<(T, String)>,
    selection: usize,
    events: Vec<AutocompleteEvent<T>>,
}

pub enum AutocompleteEvent<T> {
    Close(Option<(T, String)>),
}

impl<T> Autocomplete<T> {
    pub fn new(id: SubscriptionId, candidates: Vec<(T, String)>) -> Self {
        Self {
            id,
            candidates,
            selection: 0,
            events: Vec::new(),
        }
    }

    /// Return all autocomplete events that happened after the previous call to `events()`.
    pub fn events(&mut self) -> Vec<AutocompleteEvent<T>> {
        self.events.split_off(0)
    }
}

impl<T> Layout for Autocomplete<T>
where
    T: Clone,
{
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        ctx.grab_focus(self.id);
        ctx.subscribe(
            self.id,
            Size::ZERO.to_rect(),
            EventType::FOCUS | EventType::KEY_DOWN,
            false,
        );

        for e in ctx.events(self.id) {
            trace!("Autocomplete got event: {:?}", e);
            match e {
                Event::KeyDown(key) => {
                    if HotKey::new(None, KeyCode::Escape).matches(key) {
                        self.events.push(AutocompleteEvent::Close(None));
                    } else if HotKey::new(None, KeyCode::Return).matches(key) {
                        self.events.push(AutocompleteEvent::Close(
                            self.candidates.get(self.selection).cloned(),
                        ));
                    } else if HotKey::new(RawMods::Ctrl, KeyCode::KeyN).matches(key)
                        || HotKey::new(None, KeyCode::ArrowDown).matches(key)
                    {
                        self.selection = (self.selection + 1) % self.candidates.len();
                    } else if HotKey::new(RawMods::Ctrl, KeyCode::KeyP).matches(key)
                        || HotKey::new(None, KeyCode::ArrowUp).matches(key)
                    {
                        self.selection = self.selection.saturating_sub(1);
                    }

                    // TODO: bubble up unknown keys?
                }
                Event::Focus(_) => {}
                _ => panic!("Uknown event sent to Autocomplete, {:?}", e),
            }
        }

        let tooltip = (Color::rgb8(0xf0, 0xf0, 0xf0), Color::rgb8(0x50, 0x50, 0x50));
        let selection = (Color::rgb8(0xc0, 0xef, 0xff), Color::rgb8(0x28, 0x28, 0x28)); // TODO: Bold

        let default_width = 180.0;
        let new_min = constraint.clamp(Size::new(default_width, 0.0));

        List::new(self.candidates.iter().enumerate().map(|(i, s)| {
            let (background, foreground) = if i == self.selection {
                selection.clone()
            } else {
                tooltip.clone()
            };

            Background::new(Inset::new(
                Text::new(&s.1).with_font("Input").with_color(foreground),
                Insets::uniform_xy(0.0, 1.0),
            ))
            .with_color(background)
        }))
        .layout(ctx, Constraint::new(new_min, constraint.max))
    }
}
