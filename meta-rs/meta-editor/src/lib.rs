mod autocomplete;
mod cell_widget;
mod core_layout;
mod doc_view;
mod editor;
mod f_layout;
mod key;
mod layout;

use druid_shell::Application;

use meta_gui::{Constraint, Gui, Layout, SubscriptionId};
use meta_store::Store;

use crate::editor::Editor;

pub fn main(store: Store) {
    let app = Application::new().unwrap();
    let mut editor = Editor::new(SubscriptionId::new(), store);
    Gui::run(app.clone(), move |ctx| {
        editor.layout(ctx, Constraint::UNBOUND);
    });
    app.run(None);
}
