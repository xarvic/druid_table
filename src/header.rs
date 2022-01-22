use std::marker::PhantomData;
use druid::scroll_component::ScrollComponent;
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size, UpdateCtx, Vec2, Widget, Data, Point};
use druid::widget::{ClipBox, Scroll};
use crate::policy::HeaderTablePolicy;
use crate::Table;

struct HeaderTable<T, P: HeaderTablePolicy<T>> {
    table: Scroll<T, Table<T, P>>,

    line_header: ClipBox<T, LineHeader<T, P>>,
    line_header_size: f64,

    element_header: ClipBox<T, ElementHeader<T>>,
    element_header_size: f64,

    current_offset: Size,
}

impl<T: Data, P: HeaderTablePolicy<T>> HeaderTable<T, P> {
    fn adjust_scrolling(&mut self) {
        let offset = self.table.offset().to_size();

        if self.current_offset != offset {
            self.current_offset = offset;
            let line_axis = self.table.child().layout.line_axis();
            let line_offset = line_axis.minor(offset);
            let element_offset = line_axis.major(offset);

            self.line_header.pan_to(Point::from(line_axis.pack(0.0, line_offset)));
            self.element_header.pan_to(Point::from(line_axis.pack(element_offset, 0.0)));
        }
    }

    fn update_headers(&mut self) {

    }
}

pub struct LineHeader<T, P: HeaderTablePolicy<T>> {
    phantom: PhantomData<(T, P)>,
}

impl<T: Data, P: HeaderTablePolicy<T>> Widget<T> for LineHeader<T, P>{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        todo!()
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        todo!()
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        todo!()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        todo!()
    }
}

pub struct ElementHeader<T> {
    value: T
}

impl<T: Data> Widget<T> for ElementHeader<T>{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        todo!()
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        todo!()
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        todo!()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        todo!()
    }
}

impl<T: Data, P: HeaderTablePolicy<T>> Widget<T> for HeaderTable<T, P> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.table.event(ctx, event, data, env);
        self.adjust_scrolling();

    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {

    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        todo!()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        todo!()
    }
}
