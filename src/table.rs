use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Data, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget};
use druid::lens::Ref;
use druid::widget::{Axis, ListIter};
use crate::{Static, TableLine, TableLayout, TablePolicy, WidgetTableLine};
use crate::meta::AxisPart;

pub struct Table<T, P: TablePolicy<T>> {
    pub(crate) layout: Rc<RefCell<TableLayout>>,
    pub(crate) policy: P,
    pub(crate) lines: Vec<Box<dyn TableLine<T>>>,
}

impl<T: Data> Table<T, Static> {
    pub fn new_static(axis: Axis) -> Self {
        Self::new_dynamic(Static, axis)
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
        self.layout_mut().lines_mut().add_part(AxisPart::new(None));
        self
    }

    pub fn with_custom_line(mut self, ) {

    }
}

impl<T: Data, P: TablePolicy<T>> Table<T, P> {
    pub fn new_dynamic(policy: P, axis: Axis) -> Self {
        Self::new(policy, Rc::new(RefCell::new(TableLayout::new(axis))))
    }

    pub(crate) fn new(policy: P, layout: Rc<RefCell<TableLayout>>) -> Self {
        Self {
            layout,
            policy,
            lines: vec![]
        }
    }

    pub fn policy(&self) -> &P {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut P {
        &mut self.policy
    }

    pub fn layout_mut(&self) -> RefMut<TableLayout> {
        self.layout.deref().borrow_mut()
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

            self.layout_mut().elements_mut().set_length(elements, AxisPart::new(None));
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.policy.update(old_data, data, &mut self.lines, &mut RefCell::borrow_mut(&self.layout));
        let elements = self.lines.first().map_or(0, |line|line.element_count(data));

        for line in &mut self.lines {
            line.update(ctx, data, env);
            let line_elements = line.element_count(data);
            if line_elements != elements {
                panic!("lines of table differ in length: {} instead of {}", line_elements, elements);
            }
        }

        self.layout_mut().elements_mut().set_length(elements, AxisPart::new(None));
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let mut layout = RefCell::borrow_mut(&self.layout);
        layout.prepare_layout(bc.max());

        for (index, line) in self.lines.iter_mut().enumerate() {
            line.layout(ctx, bc, data, env, &mut *layout, index);
        }
        for (index, line) in self.lines.iter_mut().enumerate() {
            line.arrange(ctx, data, env, &mut *layout, index);
        }

        layout.table_size()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut layout = RefCell::borrow_mut(&self.layout);

        for (index, line) in self.lines.iter_mut().enumerate() {
            line.paint(ctx, data, env, &mut *layout, index);
        }
    }
}