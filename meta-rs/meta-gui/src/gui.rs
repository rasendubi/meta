use std::any::Any;
use std::time::{Duration, Instant};

use log::{trace, warn};

use druid_shell::kurbo::{Affine, Point, Rect, Shape, Size, Vec2};
use druid_shell::piet::{
    Color, Error as PietError, FontBuilder, Piet, RenderContext, Text, TextLayoutBuilder,
};
use druid_shell::{Application, KeyEvent, MouseEvent, WinHandler, WindowBuilder, WindowHandle};

pub use crate::events::{Event, EventType, SubscriptionId};
use crate::events::{EventQueue, Subscription};
use crate::ops::{Op, Ops, ShapeBox};

pub struct GuiContext<'a, 'b: 'a> {
    piet: &'a mut Piet<'b>,
    event_queue: &'a mut EventQueue,
    window_size: Size,

    ops: Ops<'b>,

    now: Instant,
}

impl<'a, 'b: 'a> GuiContext<'a, 'b> {
    fn new(piet: &'a mut Piet<'b>, event_queue: &'a mut EventQueue, window_size: Size) -> Self {
        GuiContext {
            piet,
            event_queue,
            window_size,
            ops: Ops::new(),
            now: Instant::now(),
        }
    }

    pub fn now(&self) -> Instant {
        self.now
    }

    pub fn window_size(&self) -> Size {
        self.window_size
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

    pub fn shadow(
        &mut self,
        rect: Rect,
        offset: Vec2,
        blur_radius: f64,
        spread: f64,
        brush: &<Piet as RenderContext>::Brush,
    ) {
        let rect = rect.inflate(spread, spread) + offset;
        self.blurred_rect(rect, blur_radius, brush);
    }

    pub fn clip(&mut self, shape: impl Shape) {
        self.ops.push(Op::Clip(ShapeBox::from_shape(shape)));
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

    pub fn subscribe(&mut self, id: SubscriptionId, rect: Rect, events: EventType, grab: bool) {
        self.ops.push(Op::Subscribe(Subscription {
            id,
            rect,
            events,
            grab,
        }));
    }

    pub fn grab_focus(&mut self, id: SubscriptionId) {
        self.ops.push(Op::GrabFocus(id));
    }

    pub fn events(&mut self, id: SubscriptionId) -> Vec<Event> {
        self.event_queue.take_events(id)
    }

    /// Invalidate the current frame. This leads to the current draw to not be flashed and
    /// immediately recomputed.
    ///
    /// Useful for situations when other widgets might react to the recent changes introduced by the
    /// widget.
    pub fn invalidate(&mut self) {
        trace!("invalidate()");
        self.ops.push(Op::Invalidate);
    }
}

pub struct Gui {
    handle: Option<WindowHandle>,
    ui: Box<dyn FnMut(&mut GuiContext)>,
    event_queue: EventQueue,
    window_size: Size,
    interaction: Option<Instant>,
}

impl Gui {
    pub fn run(app: Application, ui: impl FnMut(&mut GuiContext) + 'static) {
        let gui = Box::new(Gui {
            handle: None,
            ui: Box::new(ui),
            event_queue: EventQueue::new(),
            window_size: Size::ZERO,
            interaction: None,
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

    fn dispatch(&mut self, event: Event) -> bool {
        let now = Instant::now();
        let dispatched = self.event_queue.dispatch(event);
        if dispatched {
            self.interaction = Some(now);
            self.invalidate();
        }
        dispatched
    }
}

impl WinHandler for Gui {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = Some(handle.clone());
        handle.show();
        trace!("scale: {:?}", handle.get_scale());
    }

    fn paint(&mut self, piet: &mut Piet, _invalid_rect: Rect) -> bool {
        let start = Instant::now();

        let mut invalid = true;
        while invalid {
            let iteration_start = Instant::now();

            let mut ctx = GuiContext::new(piet, &mut self.event_queue, self.window_size);
            (&mut self.ui)(&mut ctx);

            let execution_start = Instant::now();
            let execution_result = ctx.ops.execute(piet);
            self.event_queue
                .replace_subscriptions(execution_result.subscriptions);

            invalid = self
                .event_queue
                .handle_grab_focus_requests(execution_result.grab_focus_requests)
                || execution_result.invalidated;

            trace!(
                target: "performance",
                "Paint iteration:{:>3}ms (ui:{:>3}ms,  ops:{:>3}ms)",
                iteration_start.elapsed().as_millis(),
                (execution_start - iteration_start).as_millis(),
                execution_start.elapsed().as_millis()
            );
        }

        trace!(target: "performance", "Paint: {:?}", start.elapsed());

        if let Some(interaction) = self.interaction.take() {
            let elapsed = interaction.elapsed();
            trace!(target: "performance", "Paint after interaction {:?}", elapsed);

            if elapsed > Duration::from_millis(100) {
                warn!("Paint after interation took {:?}", elapsed);
            }
        }

        false
    }

    fn as_any(&mut self) -> &mut dyn Any {
        todo!();
    }

    fn size(&mut self, size: Size) {
        trace!("size({:?})", size);
        self.window_size = size;
        self.invalidate();
    }

    fn wheel(&mut self, event: &MouseEvent) {
        self.dispatch(Event::MouseWheel(event.clone()));
    }

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

    fn key_down(&mut self, event: KeyEvent) -> bool {
        self.dispatch(Event::KeyDown(event))
    }

    fn key_up(&mut self, event: KeyEvent) {
        self.dispatch(Event::KeyUp(event));
    }

    fn got_focus(&mut self) {
        trace!("got_focus()");
    }
}
