use crate::gui::GuiContext;

use druid_shell::kurbo::{Point, Rect, Size, Vec2};
use druid_shell::piet::{Color, FontBuilder, RenderContext, Text, TextLayout, TextLayoutBuilder};

pub enum ButtonStyle {
    // Contained,
    // Outlined,
    Text,
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

pub fn button(ctx: &mut GuiContext, bb: Rect, s: &str) {
    let is_hovered = ctx
        .state
        .mouse
        .as_ref()
        .map_or(false, |x| bb.contains(x.pos));
    let is_clicked = is_hovered
        && ctx
            .state
            .mouse
            .as_ref()
            .map_or(false, |x| x.buttons.has_left());

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
    let text = ctx.piet.text();
    let font = text.new_font_by_name("Roboto Medium", 7.0).build().unwrap();
    let text_layout = text.new_text_layout(&font, &s, None).build().unwrap();
    let last_line = text_layout.line_metric(text_layout.line_count() - 1);
    let text_size = Size::new(
        text_layout.width(),
        last_line.as_ref().map_or(10.0, |x| x.cumulative_height),
    );
    let text_baseline = last_line.map_or(10.0, |x| x.baseline);
    let text_rect = Rect::from_center_size(bb.center(), text_size);
    let text_baseline_point = Point::new(text_rect.x0, text_rect.y0 + text_baseline - 0.5);

    // let line_baseline = text_layout.line_metric(0).map_or(9.0, |x| x.baseline);
    let text_brush = ctx.piet.solid_brush(Color::WHITE);
    ctx.piet
        .draw_text(&text_layout, text_baseline_point, &text_brush);
}
