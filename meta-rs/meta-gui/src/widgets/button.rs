use crate::gui::GuiContext;
use crate::layout::{Constraint, Layout};
use crate::widgets::clickable::Clickable;
use crate::widgets::inset::Inset;
use crate::widgets::text::Text;

use druid_shell::kurbo::{Insets, Rect, Size};
use druid_shell::piet::Color;

pub struct Button<'a> {
    clickable: &'a mut Clickable,
    text: &'a str,
}

impl<'a> Button<'a> {
    pub fn new(clickable: &'a mut Clickable, text: &'a str) -> Self {
        Button { clickable, text }
    }
}

impl<'a> Layout for Button<'a> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        // text + sizing
        let (size, ops) = ctx.capture(|ctx| {
            let s = self.text.to_uppercase();
            let mut text = Text::new(&s)
                .with_font("Roboto Medium")
                .with_size(7.0)
                .with_color(Color::WHITE);

            Inset::new(&mut text, Insets::uniform_xy(12.0, 6.0)).layout(ctx, constraint)
        });

        let bb = size.to_rect();

        self.clickable.layout(ctx, Constraint::tight(size));

        let is_hovered = self.clickable.is_hovered();
        let is_active = self.clickable.is_pressed();
        let is_clicked = is_active && is_hovered;

        // drop shadows
        button_shadows(ctx, bb, is_hovered, is_clicked);

        // button body
        let fg_color = Color::rgb8(98, 0, 238).with_alpha(
            1.0 - if is_clicked {
                0.24
            } else if is_hovered {
                0.08
            } else {
                0.0
            },
        );
        let fg_brush = ctx.solid_brush(fg_color);
        let rect = bb.to_rounded_rect(2.0);
        ctx.fill(rect, &fg_brush);

        ctx.replay(ops);

        size
    }
}

fn button_shadows(ctx: &mut GuiContext, bb: Rect, is_hovered: bool, is_active: bool) {
    let mut sh = |vo, blur, spread, color| {
        let brush = ctx.solid_brush(color);
        ctx.shadow(bb, (0.0, vo).into(), blur, spread, &brush);
    };

    if is_active {
        sh(5.0, 5.0, -3.0, Color::rgba(0.0, 0.0, 0.0, 0.2));
        sh(8.0, 10.0, 1.0, Color::rgba(0.0, 0.0, 0.0, 0.14));
        sh(3.0, 14.0, 2.0, Color::rgba(0.0, 0.0, 0.0, 0.12));
    } else if is_hovered {
        sh(2.0, 4.0, -1.0, Color::rgba(0.0, 0.0, 0.0, 0.2));
        sh(4.0, 5.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.14));
        sh(1.0, 10.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.12));
    } else {
        sh(3.0, 1.0, -2.0, Color::rgba(0.0, 0.0, 0.0, 0.2));
        sh(2.0, 2.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.14));
        sh(1.0, 5.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.12));
    }
}
