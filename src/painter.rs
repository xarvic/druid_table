use druid::{Env, PaintCtx, Data, RenderContext};
use druid::kurbo::Line;
use druid::theme::BORDER_LIGHT;
use druid::widget::Axis;
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

        let size = ctx.size();
        let brush = env.get(BORDER_LIGHT);

        let mut advance_x = 0.5;
        let mut iter_x = layout.direction_axis(Axis::Vertical).parts().iter();
        loop {
            ctx.stroke(Line::new((advance_x, 0.5), (advance_x, size.height + 0.5)), &brush, 1.0);

            if let Some(part) = iter_x.next() {
                advance_x += part.advance();
            } else {
                break;
            }
        }

        let mut advance_y = 0.5;
        let mut iter_y = layout.direction_axis(Axis::Horizontal).parts().iter();
        loop {
            ctx.stroke(Line::new((0.5, advance_y), (size.width + 0.5, advance_y)), &brush, 1.0);

            if let Some(part) = iter_y.next() {
                advance_y += part.advance();
            } else {
                break;
            }
        }
    }
}