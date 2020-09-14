use std::any::Any;
use std::time::Instant;

use log::trace;

use druid_shell::kurbo::{Affine, Point, Rect, Shape, Size};
use druid_shell::piet::{
    Color, Error as PietError, FontBuilder, Piet, RenderContext, Text, TextLayoutBuilder,
};
use druid_shell::{Application, MouseEvent, WinHandler, WindowBuilder, WindowHandle};

pub use crate::events::{Event, EventType, WidgetId};
use crate::events::{EventQueue, Subscription};
use crate::ops::{Op, Ops, ShapeBox};

pub struct GuiContext<'a, 'b: 'a> {
    piet: &'a mut Piet<'b>,
    event_queue: &'a mut EventQueue,

    ops: Ops<'b>,

    now: Instant,
    key_stack: Vec<String>,
}

impl<'a, 'b: 'a> GuiContext<'a, 'b> {
    fn new(piet: &'a mut Piet<'b>, event_queue: &'a mut EventQueue) -> Self {
        GuiContext {
            piet,
            event_queue,
            ops: Ops::new(),
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

    pub fn solid_brush(&mut self, color: Color) -> <Piet as RenderContext>::Brush {
        self.piet.solid_brush(color)
    }

    pub fn new_font_by_name(
        &mut self,
        name: &str,
        size: f64,
    ) -> Result<<<Piet as RenderContext>::Text as Text>::Font, PietError> {
        self.piet.text().new_font_by_name(name, size).build()
    }

    pub fn new_text_layout(
        &mut self,
        font: &<<Piet as RenderContext>::Text as Text>::Font,
        text: &str,
        width: impl Into<Option<f64>>,
    ) -> Result<<<Piet as RenderContext>::Text as Text>::TextLayout, PietError> {
        self.piet.text().new_text_layout(font, text, width).build()
    }

    pub fn draw_text(
        &mut self,
        layout: &<<Piet as RenderContext>::Text as Text>::TextLayout,
        pos: impl Into<Point>,
        brush: &<Piet as RenderContext>::Brush,
    ) {
        let pos = pos.into();
        self.ops.push(Op::SetBrush(brush.clone()));
        self.ops.push(Op::DrawText {
            layout: layout.clone(),
            pos,
        });
    }

    pub fn blurred_rect(
        &mut self,
        rect: Rect,
        blur_radius: f64,
        brush: &<Piet as RenderContext>::Brush,
    ) {
        self.ops.push(Op::SetBrush(brush.clone()));
        self.ops.push(Op::BlurredRect { rect, blur_radius });
    }

    pub fn clear(&mut self, color: Color) {
        self.ops.push(Op::Clear(color));
    }

    pub fn fill(&mut self, shape: impl Shape, brush: &<Piet as RenderContext>::Brush) {
        self.ops.push(Op::SetBrush(brush.clone()));
        self.ops.push(Op::Fill(ShapeBox::from_shape(shape)));
    }

    pub fn transform(&mut self, transform: Affine) {
        self.ops.push(Op::Transform(transform));
    }

    pub fn with_save<F: FnOnce(&mut Self) -> R, R>(&mut self, f: F) -> R {
        self.ops.push(Op::Save);
        let r = f(self);
        self.ops.push(Op::Restore);
        r
    }

    pub fn capture<F: FnOnce(&mut Self) -> R, R>(&mut self, f: F) -> (R, Ops<'b>) {
        let mut ops = Ops::<'b>::new();
        std::mem::swap(&mut self.ops, &mut ops);
        let r = f(self);
        std::mem::swap(&mut self.ops, &mut ops);
        (r, ops)
    }

    pub fn replay(&mut self, ops: Ops<'b>) {
        self.ops.push_all(ops);
    }

    pub fn subscribe(&mut self, rect: Rect, events: EventType, grab: bool) {
        let widget_id = self.get_widget_id();
        self.ops.push(Op::Subscribe(Subscription {
            widget_id,
            rect,
            events,
            grab,
        }));
    }

    pub fn events(&mut self) -> Vec<Event> {
        let widget_id = self.get_widget_id();
        self.event_queue.widget_events(widget_id)
    }
}

pub struct Gui {
    handle: Option<WindowHandle>,
    ui: Box<dyn FnMut(&mut GuiContext)>,
    event_queue: EventQueue,
}

impl Gui {
    pub fn run(app: Application, ui: impl FnMut(&mut GuiContext) + 'static) {
        let gui = Box::new(Gui {
            handle: None,
            ui: Box::new(ui),
            event_queue: EventQueue::new(),
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

    fn dispatch(&mut self, event: Event) {
        if self.event_queue.dispatch(event) {
            self.invalidate();
        }
    }
}

impl WinHandler for Gui {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = Some(handle.clone());
        handle.show();
    }

    fn paint(&mut self, piet: &mut Piet, _invalid_rect: Rect) -> bool {
        let start = std::time::Instant::now();

        let mut ctx = GuiContext::new(piet, &mut self.event_queue);
        (&mut self.ui)(&mut ctx);
        let subscriptions = ctx.ops.execute(piet);
        self.event_queue.replace_subscriptions(subscriptions);

        trace!("Paint done in {:?}", start.elapsed());
        false
    }

    fn as_any(&mut self) -> &mut dyn Any {
        todo!();
    }

    fn size(&mut self, _size: Size) {
        // TODO: handle size
        self.invalidate();
    }

    fn wheel(&mut self, _event: &MouseEvent) {}

    fn mouse_move(&mut self, event: &MouseEvent) {
        self.dispatch(Event::MouseMove(event.clone()));
    }

    fn mouse_down(&mut self, event: &MouseEvent) {
        self.dispatch(Event::MouseDown(event.clone()));
    }

    fn mouse_up(&mut self, event: &MouseEvent) {
        self.dispatch(Event::MouseUp(event.clone()));
    }

    fn mouse_leave(&mut self) {
        self.dispatch(Event::MouseLeave);
    }
}
