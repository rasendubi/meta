use druid_shell::kurbo::{Insets, Size};
use druid_shell::piet::Color;
use druid_shell::{HotKey, KeyCode, RawMods};
use log::trace;
use unicode_segmentation::UnicodeSegmentation;

use meta_gui::{
    Background, Column, Constraint, Event, EventType, GuiContext, Inset, Layout, List,
    SubscriptionId, Text,
};

pub struct Autocomplete<T> {
    id: SubscriptionId,
    candidates: Vec<(T, String)>,
    selection: usize,
    events: Vec<AutocompleteEvent<T>>,
    input: String,
}

pub enum AutocompleteEvent<T> {
    Close(Option<(T, String)>),
    InputChanged(String),
}

impl<T> Autocomplete<T> {
    pub fn new(id: SubscriptionId, candidates: Vec<(T, String)>) -> Self {
        Self {
            id,
            candidates,
            selection: 0,
            events: Vec::new(),
            input: String::new(),
        }
    }

    /// Return all autocomplete events that happened after the previous call to `events()`.
    pub fn events(&mut self) -> Vec<AutocompleteEvent<T>> {
        self.events.split_off(0)
    }

    pub fn with_input(mut self, input: String) -> Self {
        self.input = input;
        self
    }

    pub fn set_candidates(&mut self, candidates: Vec<(T, String)>) {
        self.candidates = candidates;
        self.selection = self.selection.min(self.candidates.len().saturating_sub(1));
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
                        || HotKey::new(None, KeyCode::Tab).matches(key)
                    {
                        self.selection = (self.selection + 1) % self.candidates.len();
                    } else if HotKey::new(RawMods::Ctrl, KeyCode::KeyP).matches(key)
                        || HotKey::new(None, KeyCode::ArrowUp).matches(key)
                        || HotKey::new(RawMods::Shift, KeyCode::Tab).matches(key)
                    {
                        self.selection = self.selection.saturating_sub(1);
                    } else if HotKey::new(None, KeyCode::Backspace).matches(key) {
                        let idx = self.input.grapheme_indices(true).last().map(|x| x.0);
                        if let Some(idx) = idx {
                            self.input.remove(idx);
                            self.events
                                .push(AutocompleteEvent::InputChanged(self.input.clone()));
                        }
                    } else if let Some(text) = key.text() {
                        if !key.mods.alt
                            && !key.mods.ctrl
                            && !key.mods.meta
                            && text.chars().all(|c| !c.is_control())
                        {
                            self.input.push_str(text);
                            self.events
                                .push(AutocompleteEvent::InputChanged(self.input.clone()));
                        }
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

        let mut minput = if self.input.is_empty() {
            None
        } else {
            Some(&self.input)
        }
        .map(|x| {
            Box::new(
                Background::new(
                    Text::new(x.clone())
                        .with_font("Input")
                        .with_color(Color::rgb8(0x0a, 0x0a, 0x0a)),
                )
                .with_color(Color::rgb8(0xd7, 0xd7, 0xd7)),
            )
        });

        let (size, ops) = ctx.capture(|ctx| {
            Column::new()
                .with_child(&mut minput as &mut dyn Layout)
                .with_child(&mut List::new(self.candidates.iter().enumerate().map(
                    |(i, s)| {
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
                    },
                )))
                .layout(ctx, Constraint::new(new_min, constraint.max))
        });

        let shadow_brush = ctx.solid_brush(Color::rgba(0.0, 0.0, 0.0, 0.2));
        ctx.shadow(size.to_rect(), (0.0, 2.0).into(), 2.0, 0.0, &shadow_brush);
        // Each candidate draws its own background, but this produces small holes between some
        // candidates (because of rounding errors?). Draw an extra background to fix that.
        let background_brush = ctx.solid_brush(tooltip.0);
        ctx.fill(size.to_rect(), &background_brush);

        ctx.replay(ops);
        size
    }
}
