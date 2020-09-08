use meta_gui::{button, Gui, GuiContext, Text, TextPosition};

use druid_shell::kurbo::{Point, Rect, Size};
use druid_shell::piet::{Color, RenderContext};
use druid_shell::Application;

fn main() {
    let app = Application::new().unwrap();
    Gui::run(app.clone(), ui);
    app.run(None);
}

fn ui(ctx: &mut GuiContext) {
    ctx.piet.clear(Color::WHITE);

    Text::new("Hello, world!")
        .with_position(TextPosition::TopLeft(Point::new(0.0, 0.0)))
        .with_font("Input")
        .draw(ctx);

    Text::new("Almost before we knew it, we had left the ground.")
        .with_position(TextPosition::TopLeft(Point::new(0.0, 12.0)))
        .draw(ctx);

    ctx.with_key(&"button1", |ctx| {
        if button(
            ctx,
            Rect::from_origin_size(Point::new(40.0, 40.0), Size::new(94.0 / 2.0, 36.0 / 2.0)),
            "Button1",
        ) {
            println!("button1 clicked");
        }
    });
    ctx.with_key(&"button2", |ctx| {
        if button(
            ctx,
            Rect::from_origin_size(Point::new(40.0, 60.0), Size::new(94.0 / 2.0, 36.0 / 2.0)),
            "Button2",
        ) {
            println!("button2 clicked");
        }
    });
}
