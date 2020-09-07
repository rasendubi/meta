use std::any::Any;
use std::time::Instant;

use druid_shell::kurbo::{Point, Rect, Size};
use druid_shell::piet::Piet;
use druid_shell::{Application, MouseEvent, WinHandler, WindowBuilder, WindowHandle};

#[derive(Debug)]
pub struct GuiState {
    pub size: Size,
    /// Mouse position
    pub mouse: Option<(Point, Instant)>,
    /// The mouse down event
    pub mouse_left_down: Option<(MouseEvent, Instant)>,
    /// Widget that has been clicked
    pub active_widget: Option<WidgetId>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct WidgetId(u64);

pub struct GuiContext<'a, 'b: 'a> {
    pub piet: &'a mut Piet<'b>,
    pub state: &'a mut GuiState,

    now: Instant,
    key_stack: Vec<String>,
}

impl<'a, 'b: 'a> GuiContext<'a, 'b> {
    fn new(piet: &'a mut Piet<'b>, state: &'a mut GuiState) -> Self {
        GuiContext {
            piet,
            state,
            key_stack: Vec::new(),
            now: Instant::now(),
        }
    }

    pub fn with_key<F, R>(&mut self, key: &impl ToString, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.key_stack.push(key.to_string());
        let r = f(self);
        self.key_stack.pop();
        r
    }

    pub fn get_widget_id(&self) -> WidgetId {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.key_stack.hash(&mut hasher);
        WidgetId(hasher.finish())
    }

    pub fn now(&self) -> Instant {
        self.now
    }
}

pub struct Gui {
    handle: Option<WindowHandle>,
    ui: Box<dyn Fn(&mut GuiContext)>,
    state: GuiState,
}

impl Gui {
    pub fn run(app: Application, ui: impl Fn(&mut GuiContext) + 'static) {
        let gui = Box::new(Gui {
            handle: None,
            ui: Box::new(ui),
            state: GuiState {
                size: Size::default(),
                mouse: None,
                mouse_left_down: None,
                active_widget: None,
            },
        });

        let mut window_builder = WindowBuilder::new(app);
        window_builder.set_handler(gui);
        let _window = window_builder.build().unwrap();
    }
}

impl Gui {
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
        // let start = std::time::Instant::now();

        let mut ctx = GuiContext::new(piet, &mut self.state);
        // println!("Paint context: {:?}", ctx.state);
        (&self.ui)(&mut ctx);
        // println!("Paint done in {:?}", start.elapsed());

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
        // println!("wheel: {:?}", event);
        self.state.mouse = Some((event.pos, Instant::now()));
        self.invalidate();
    }

    fn mouse_move(&mut self, event: &MouseEvent) {
        // println!("mouse_move: {:?}", event);
        self.state.mouse = Some((event.pos, Instant::now()));
        self.invalidate();
    }

    fn mouse_down(&mut self, event: &MouseEvent) {
        // println!("mouse_down: {:?}", event);
        let now = Instant::now();
        self.state.mouse = Some((event.pos, now));
        if self.state.mouse_left_down.is_none() && event.button.is_left() {
            self.state.mouse_left_down = Some((event.clone(), now));
        }
        self.invalidate();
    }

    fn mouse_up(&mut self, event: &MouseEvent) {
        // println!("mouse_up: {:?}", event);
        self.state.mouse = Some((event.pos, Instant::now()));
        if event.button.is_left() {
            self.state.mouse_left_down = None;
        }
        self.invalidate();
    }

    fn mouse_leave(&mut self) {
        // println!("mouse_leave");
        self.state.mouse = None;
        self.invalidate();
    }
}