use meta_gui::{
    Button, Clickable, Column, Constraint, Gui, GuiContext, Inset, Layout, Row, SubscriptionId,
    Text,
};

use druid_shell::kurbo::Insets;
use druid_shell::piet::Color;
use druid_shell::Application;

fn main() {
    let app = Application::new().unwrap();

    let mut ui = Ui {
        click1: Clickable::new(SubscriptionId::new()),
        click2: Clickable::new(SubscriptionId::new()),
    };

    Gui::run(app.clone(), move |ctx| ui.draw(ctx));
    app.run(None);
}

struct Ui {
    click1: Clickable,
    click2: Clickable,
}

impl Ui {
    fn draw(&mut self, ctx: &mut GuiContext) {
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
                &mut Button::new(&mut self.click1, "button1"),
                Insets::uniform_xy(2.0, 4.0),
            ))
            .with_child(&mut Inset::new(
                &mut Button::new(&mut self.click2, "button2"),
                Insets::uniform_xy(2.0, 4.0),
            ))
            .layout(ctx, Constraint::unbound());

        for _ in self.click1.clicks() {
            println!("Click1");
        }
        for _ in self.click2.clicks() {
            println!("Click2");
        }
    }
}
