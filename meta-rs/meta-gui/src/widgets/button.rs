use crate::gui::GuiContext;

use crate::layout::{Constraint, Layout};
use crate::widgets::center::Center;
use crate::widgets::text::Text;

use druid_shell::kurbo::{Rect, Vec2};
use druid_shell::piet::{Color, RenderContext};

pub struct ButtonState {
    /// Mouse is over the button.
    pub is_hovered: bool,
    /// The button has been pushed down.
    pub is_active: bool,
    /// The button has been clicked and is released now.
    pub click: bool,
}

/// Calculate button interaction with mouse position and left button.
///
/// Useful for button-like behavior.
pub fn button_behavior(ctx: &mut GuiContext, bb: Rect) -> ButtonState {
    let widget_id = ctx.get_widget_id();

    let is_hovered = ctx.state.mouse.as_ref().map_or(false, |x| bb.contains(x.0));

    if ctx.state.mouse_left_down.is_some() && ctx.state.active_widget.is_none() {
        let (left, _) = ctx.state.mouse_left_down.as_ref().unwrap();
        if bb.contains(left.pos) {
            // we have been clicked
            ctx.state.active_widget = Some(widget_id);
        }
    }

    let is_active = ctx.state.active_widget == Some(widget_id);

    let click = if is_active && ctx.state.mouse_left_down.is_none() {
        ctx.state.active_widget = None;

        is_hovered
    } else {
        false
    };

    // update is_active after we calculate click (it might have changed)
    let is_active = ctx.state.active_widget == Some(widget_id);

    ButtonState {
        is_hovered,
        is_active,
        click,
    }
}

pub enum ButtonStyle {
    Contained,
    // Outlined,
    // Text,
}

fn shadow(
    ctx: &mut GuiContext,
    bb: Rect,
    horizontal_offset: f64,
    vertical_offset: f64,
    blur_radius: f64,
    spread: f64,
    color: Color,
) {
    let brush = ctx.piet.solid_brush(color);
    let rect = bb.inflate(spread, spread) + Vec2::new(horizontal_offset, vertical_offset);
    ctx.piet.blurred_rect(rect, blur_radius, &brush);
}

fn button_shadows(ctx: &mut GuiContext, bb: Rect, is_hovered: bool, is_active: bool) {
    let mut sh = |vo, blur, spread, color| shadow(ctx, bb, 0.0, vo, blur, spread, color);

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

pub fn button(ctx: &mut GuiContext, bb: Rect, s: &str) -> bool {
    let ButtonState {
        is_hovered,
        is_active,
        click,
    } = button_behavior(ctx, bb);
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
    let fg_brush = ctx.piet.solid_brush(fg_color);
    let rect = bb.to_rounded_rect(2.0);
    ctx.piet.fill(rect, &fg_brush);

    // text
    let s = s.to_uppercase();
    let mut text = Text::new(&s)
        .with_font("Roboto Medium")
        .with_size(7.0)
        .with_color(Color::WHITE);

    let mut center = Center::new(&mut text);
    center.set_constraint(ctx, Constraint::tight(bb.size()));
    center.set_origin(bb.origin());

    text.draw(ctx);

    click
}
