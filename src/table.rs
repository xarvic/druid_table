use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Data, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget};
use druid::widget::{Axis, ListIter};
use crate::{Static, TableLine, TableLayout, TablePolicy, WidgetTableLine};

pub struct Table<T, P: TablePolicy<T>> {
    pub(crate) layout: TableLayout,
    pub(crate) policy: P,
    pub(crate) lines: Vec<Box<dyn TableLine<T>>>,
}

impl<T: Data> Table<T, Static> {
    pub fn new_static(axis: Axis) -> Self {
        Self {
            layout: TableLayout::new(axis),
            policy: Static,
            lines: vec![]
        }
    }

    pub fn with_line<
        T2: ListIter<U> + Data,
        U: Data,
        V: Data,

        L1: Lens<T, T2> + 'static,
        L2: Lens<U, V> + 'static,
        W: Widget<V> + 'static,
        F: Fn() -> W + 'static,
    >(mut self, outer_lens: L1, inner_lens: L2, widget: F) -> Self {
        self.lines.push(Box::new(WidgetTableLine::new(outer_lens, inner_lens, widget)));
        self.layout.add_line(None);
        self
    }


}

impl<T: Data, P: TablePolicy<T>> Widget<T> for Table<T, P> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for line in &mut self.lines {
            line.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for line in &mut self.lines {
            line.lifecycle(ctx, event, data, env);
        }

        if let LifeCycle::WidgetAdded = event {
            let elements = self.lines.first().map_or(0, |line|line.element_count(data));
            for line in &mut self.lines {
                let line_elements = line.element_count(data);
                if line_elements != elements {
                    panic!("lines of table differ in length: {} instead of {}", line_elements, elements);
                }
            }

            self.layout.set_element_count(elements, None);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.policy.update(old_data, data, &mut self.lines, &mut self.layout);
        let elements = self.lines.first().map_or(0, |line|line.element_count(data));

        for line in &mut self.lines {
            line.update(ctx, data, env);
            let line_elements = line.element_count(data);
            if line_elements != elements {
                panic!("lines of table differ in length: {} instead of {}", line_elements, elements);
            }
        }

        self.layout.set_element_count(elements, None);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.layout.prepare_layout();

        for (index, line) in self.lines.iter_mut().enumerate() {
            line.layout(ctx, bc, data, env, &mut self.layout, index);
        }
        for (index, line) in self.lines.iter_mut().enumerate() {
            line.arrange(ctx, data, env, &mut self.layout, index);
        }

        self.layout.table_size()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for (index, line) in self.lines.iter_mut().enumerate() {
            line.paint(ctx, data, env, &mut self.layout, index);
        }
    }
}