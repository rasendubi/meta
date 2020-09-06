use meta_gui::{Gui, GuiContext};

use druid_shell::kurbo::{Point, Rect};
use druid_shell::piet::{Color, FontBuilder, RenderContext, Text, TextLayout, TextLayoutBuilder};
use druid_shell::{Application, WindowBuilder};

fn main() {
    let app = Application::new().unwrap();
    let mut window_builder = WindowBuilder::new(app.clone());
    window_builder.set_handler(Box::new(Gui::new(ui)));
    let _window = window_builder.build().unwrap();

    app.run(None);
}

fn ui(ctx: &mut GuiContext) {
    let piet = &mut ctx.piet;

    piet.clear(Color::WHITE);
    let text = piet.text();
    let font = text.new_font_by_name("Input", 9.0).build().unwrap();
    let text_layout = text
        .new_text_layout(&font, "Hello, world!", None)
        .build()
        .unwrap();
    let line_baseline = text_layout.line_metric(0).map_or(9.0, |x| x.baseline);
    let brush = piet.solid_brush(Color::BLACK);
    piet.draw_text(&text_layout, Point::new(0.0, line_baseline), &brush);

    piet.blurred_rect(Rect::new(40.0, 40.0, 80.0, 80.0), 3.0, &brush);
    let brush_blue = piet.solid_brush(Color::rgb(0, 0, 255));
    piet.fill(Rect::new(40.0, 40.0, 80.0, 80.0), &brush_blue);
}
