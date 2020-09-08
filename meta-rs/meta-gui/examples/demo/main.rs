use meta_gui::{button, Column, Constraint, Gui, GuiContext, Layout, Row, Text};

use druid_shell::kurbo::{Point, Rect, Size};
use druid_shell::piet::Color;
use druid_shell::Application;

fn main() {
    let app = Application::new().unwrap();
    Gui::run(app.clone(), ui);
    app.run(None);
}

fn ui(ctx: &mut GuiContext) {
    ctx.clear(Color::WHITE);

    Column::new()
        .with_child(&mut Text::new("Hello, world!").with_font("Input"))
        .with_child(&mut Text::new(
            "Almost before we knew it, we had left the ground.",
        ))
        .with_child(
            &mut Row::new()
                .with_child(&mut Text::new("Hello, "))
                .with_child(&mut Text::new("world!")),
        )
        .layout(ctx, Constraint::unbound());

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
