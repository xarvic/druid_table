use druid::{Env, Event, EventCtx, LifeCycle, LifeCycleCtx, UpdateCtx, Data};
use crate::table::TableContent;
use crate::TableLayout;

pub trait TableController<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env, content: &mut TableContent<T>, layout: &TableLayout);

    #[allow(unused_variables)]
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {}

    #[allow(unused_variables)]
    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {}
}

pub struct DefaultTableController;

impl<T: Data> TableController<T> for DefaultTableController {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env, content: &mut TableContent<T>, _: &TableLayout) {
        content.event(ctx, event, data, env)
    }
}