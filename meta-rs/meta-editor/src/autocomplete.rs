use druid_shell::{
    kurbo::{Insets, Size},
    piet::Color,
};
use meta_gui::{Background, Constraint, GuiContext, Inset, Layout, List, Text};

pub struct Autocomplete<'a> {
    candidates: &'a [&'a str],
}

impl<'a> Autocomplete<'a> {
    pub fn new(candidates: &'a [&'a str]) -> Self {
        Self { candidates }
    }
}

impl<'a> Layout for Autocomplete<'a> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let tooltip = (Color::rgb8(0xf0, 0xf0, 0xf0), Color::rgb8(0x50, 0x50, 0x50));
        let selection = (Color::rgb8(0xc0, 0xef, 0xff), Color::rgb8(0x28, 0x28, 0x28)); // Bold

        let default_width = 180.0;
        let new_min = constraint.clamp(Size::new(default_width, 0.0));

        List::new(self.candidates.iter().enumerate().map(|(i, s)| {
            let is_selection = i == 0;
            let (background, foreground) = if is_selection {
                selection.clone()
            } else {
                tooltip.clone()
            };

            Background::new(Inset::new(
                Text::new(s).with_font("Input").with_color(foreground),
                Insets::uniform(1.0),
            ))
            .with_color(background)
        }))
        .layout(ctx, Constraint::new(new_min, constraint.max))
    }
}
