pub mod core_layout;
pub mod layout;

use druid::theme::{LABEL_COLOR, WINDOW_BACKGROUND_COLOR};
use druid::widget::{Label, Padding, Scroll};
use druid::{AppLauncher, Color, PlatformError, Widget, WindowDesc};

use crate::core_layout::core_layout_entities;
use crate::layout::simple_doc_to_string;
use meta_core::MetaCore;
use meta_store::MetaStore;

pub fn main(store: MetaStore) -> Result<(), PlatformError> {
    let core = MetaCore::new(&store);
    let rich_doc = core_layout_entities(&core);
    let layout = meta_pretty::layout(&rich_doc, 80);
    let s = simple_doc_to_string(&layout);

    println!("{}", s);

    let main_window = WindowDesc::new(ui_builder);
    AppLauncher::with_window(main_window)
        .configure_env(|env, _state| {
            env.set(WINDOW_BACKGROUND_COLOR, Color::WHITE);
            env.set(LABEL_COLOR, Color::BLACK);
        })
        .use_simple_logger()
        .launch(s)
}

fn ui_builder() -> impl Widget<String> {
    let label = Label::dynamic(|text: &String, _| text.to_string())
        .with_text_size(9.0)
        .with_font("Input".to_string());

    Scroll::new(Padding::new(10.0, label))
}
