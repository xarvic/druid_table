use druid::{Env, PaintCtx, Data};
use crate::table::TableContent;
use crate::TableLayout;

pub trait TablePainter<T> {
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env, content: &mut TableContent<T>, layout: &TableLayout);
}

pub struct DefaultTablePainter;

impl<T: Data> TablePainter<T> for DefaultTablePainter {
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env, content: &mut TableContent<T>, layout: &TableLayout) {
        content.paint_background(ctx, data, env, layout);
        content.paint_foreground(ctx, data, env, layout);
    }
}