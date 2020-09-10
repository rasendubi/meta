use crate::gui::GuiContext;
use crate::layout::*;

use druid_shell::kurbo::Size;

pub struct WithKey<'a> {
    key: String,
    child: &'a mut dyn Layout,
}

impl<'a> WithKey<'a> {
    pub fn new(key: &impl ToString, child: &'a mut dyn Layout) -> Self {
        WithKey {
            key: key.to_string(),
            child,
        }
    }
}

impl<'a> Layout for WithKey<'a> {
    fn layout(&mut self, ctx: &mut GuiContext, constraint: Constraint) -> Size {
        let child = &mut self.child; // to please borrow-checker gods
        ctx.with_key(&self.key, |ctx| child.layout(ctx, constraint))
    }
}
