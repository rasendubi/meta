pub mod core_layout;
pub mod layout;

use druid_shell::kurbo::Size;
use druid_shell::piet::Color;
use druid_shell::Application;
use meta_gui::{Constraint, Direction, Gui, GuiContext, Layout, List, Text};

use crate::core_layout::core_layout_entities;
use crate::layout::EditorCellPayload;
use meta_core::MetaCore;
use meta_pretty::SimpleDoc;
use meta_store::MetaStore;

pub fn main(store: MetaStore) {
    let core = MetaCore::new(&store);
    let rich_doc = core_layout_entities(&core);
    let sdoc = meta_pretty::layout(&rich_doc, 80);
    let layout = layout_to_2d(sdoc);

    let app = Application::new().unwrap();

    let mut editor = Editor { layout };

    Gui::run(app.clone(), move |ctx| editor.run(ctx));
    app.run(None);
}

struct Editor {
    layout: Vec<Vec<SimpleDoc<EditorCellPayload>>>,
}

impl Editor {
    pub fn run(&mut self, ctx: &mut GuiContext) {
        ctx.clear(Color::WHITE);

        List::new(self.layout.iter().map(|line| {
            List::new(line.iter().map(CellWidget)).with_direction(Direction::Horizontal)
        }))
        .layout(ctx, Constraint::unbound());
    }
}

struct CellWidget<'a>(&'a SimpleDoc<EditorCellPayload>);

impl<'a> Layout for CellWidget<'a> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        match self.0 {
            SimpleDoc::Cell(cell) => Text::new(cell.payload.text.as_ref())
                .with_font("Input")
                .layout(ctx, constraint),
            SimpleDoc::Linebreak { indent_width } => {
                let mut s = String::with_capacity(*indent_width);
                for _ in 0..*indent_width {
                    s.push(' ');
                }
                Text::new(&s).with_font("Input").layout(ctx, constraint)
            }
        }
    }
}

fn layout_to_2d<T>(layout: Vec<SimpleDoc<T>>) -> Vec<Vec<SimpleDoc<T>>> {
    let mut result = vec![Vec::new()];

    for cell in layout.into_iter() {
        if let SimpleDoc::Linebreak { .. } = cell {
            result.push(Vec::new());
        }

        let last = result.len() - 1;
        unsafe {
            result.get_unchecked_mut(last).push(cell);
        }
    }

    result
}
