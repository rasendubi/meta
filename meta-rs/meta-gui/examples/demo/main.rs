use meta_gui::{Button, Column, Constraint, Gui, GuiContext, Inset, Layout, Row, Text, WithKey};

use druid_shell::kurbo::Insets;
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
        .with_child(&mut Inset::new(
            &mut WithKey::new(&"button1", &mut Button::new("button1")),
            Insets::uniform_xy(2.0, 4.0),
        ))
        .with_child(&mut Inset::new(
            &mut WithKey::new(&"button2", &mut Button::new("button2")),
            Insets::uniform_xy(2.0, 4.0),
        ))
        .layout(ctx, Constraint::unbound());
}
