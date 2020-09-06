use std::any::Any;

use druid_shell::kurbo::{Rect, Size};
use druid_shell::piet::Piet;
use druid_shell::{Application, MouseEvent, WinHandler, WindowBuilder, WindowHandle};

pub struct GuiContext<'a, 'b: 'a> {
    pub piet: &'a mut Piet<'b>,
    pub state: &'a GuiState,
}

pub struct Gui {
    handle: Option<WindowHandle>,
    ui: Box<dyn Fn(&mut GuiContext)>,
    state: GuiState,
}

pub struct GuiState {
    pub size: Size,
    pub mouse: Option<MouseEvent>,
}

impl Gui {
    pub fn run(app: Application, ui: impl Fn(&mut GuiContext) + 'static) {
        let gui = Box::new(Gui {
            handle: None,
            ui: Box::new(ui),
            state: GuiState {
                size: Size::default(),
                mouse: None,
            },
        });

        let mut window_builder = WindowBuilder::new(app);
        window_builder.set_handler(gui);
        let _window = window_builder.build().unwrap();
    }
}

impl Gui {
    fn set_mouse(&mut self, event: &MouseEvent) {
        self.state.mouse = Some(event.clone());
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.handle.as_mut().unwrap().invalidate();
    }
}

impl WinHandler for Gui {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = Some(handle.clone());
        handle.show();
    }

    fn paint(&mut self, piet: &mut Piet, _invalid_rect: Rect) -> bool {
        let start = std::time::Instant::now();

        let mut ctx = GuiContext {
            piet,
            state: &self.state,
        };
        (&self.ui)(&mut ctx);

        println!("Paint done in {:?}", start.elapsed());

        false
    }

    fn as_any(&mut self) -> &mut dyn Any {
        todo!();
    }

    fn size(&mut self, size: Size) {
        self.state.size = size;
        self.invalidate();
    }

    fn wheel(&mut self, event: &MouseEvent) {
        self.set_mouse(event);
    }

    fn mouse_move(&mut self, event: &MouseEvent) {
        self.set_mouse(event);
    }

    fn mouse_down(&mut self, event: &MouseEvent) {
        self.set_mouse(event);
    }

    fn mouse_up(&mut self, event: &MouseEvent) {
        self.set_mouse(event);
    }

    fn mouse_leave(&mut self) {
        self.state.mouse = None;
        self.invalidate();
    }
}
