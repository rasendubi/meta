use std::any::Any;

use druid_shell::kurbo::Rect;
use druid_shell::piet::Piet;
use druid_shell::{WinHandler, WindowHandle};

pub struct GuiContext<'a, 'b: 'a> {
    pub piet: &'a mut Piet<'b>,
}

pub struct Gui {
    ui: Box<dyn Fn(&mut GuiContext)>,
}

impl Gui {
    pub fn new(ui: impl Fn(&mut GuiContext) + 'static) -> Self {
        Gui { ui: Box::new(ui) }
    }
}

impl WinHandler for Gui {
    fn connect(&mut self, handle: &WindowHandle) {
        handle.show();
    }

    fn paint(&mut self, piet: &mut Piet, _invalid_rect: Rect) -> bool {
        let mut ctx = GuiContext { piet };
        (&self.ui)(&mut ctx);
        false
    }

    fn as_any(&mut self) -> &mut dyn Any {
        todo!();
    }
}
