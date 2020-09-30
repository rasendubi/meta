mod cell_widget;
mod core_layout;
mod editor;
mod layout;

use druid_shell::Application;
use meta_gui::{Constraint, Gui, Layout, SubscriptionId};

use crate::editor::Editor;
use meta_store::MetaStore;

pub fn main(store: MetaStore) {
    let app = Application::new().unwrap();
    let mut editor = Editor::new(SubscriptionId::new(), store);
    Gui::run(app.clone(), move |ctx| {
        editor.layout(ctx, Constraint::unbound());
    });
    app.run(None);
}
