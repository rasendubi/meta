use meta_gui::{button, Gui, GuiContext};

use druid_shell::kurbo::{Point, Rect, Size};
use druid_shell::piet::{Color, FontBuilder, RenderContext, Text, TextLayout, TextLayoutBuilder};
use druid_shell::Application;

fn main() {
    let app = Application::new().unwrap();
    Gui::run(app.clone(), ui);
    app.run(None);
}

fn ui(ctx: &mut GuiContext) {
    ctx.piet.clear(Color::WHITE);

    let text = ctx.piet.text();
    let font = text.new_font_by_name("Input", 9.0).build().unwrap();
    let text_layout = text
        .new_text_layout(&font, "Hello, world!", None)
        .build()
        .unwrap();
    let line_baseline = text_layout.line_metric(0).map_or(9.0, |x| x.baseline);
    let brush = ctx.piet.solid_brush(Color::BLACK);
    ctx.piet
        .draw_text(&text_layout, Point::new(0.0, line_baseline), &brush);

    button(
        ctx,
        Rect::from_origin_size(Point::new(40.0, 40.0), Size::new(94.0 / 2.0, 36.0 / 2.0)),
        "Button",
    );

    // mouse_rect(ctx);
}

#[allow(unused)]
fn mouse_rect(ctx: &mut GuiContext) {
    if let Some(mouse) = &ctx.state.mouse {
        let rect = Rect::from_center_size(mouse.pos, Size::new(10.0, 10.0));
        let brush = ctx.piet.solid_brush(Color::rgb(0, 255, 0));
        ctx.piet.fill(rect, &brush);
    }
}
