use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Data, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget};
use druid::lens::Ref;
use druid::widget::{Axis, ListIter};
use crate::{Static, TableLine, TableLayout, TablePolicy, WidgetTableLine, DefaultTableController, DefaultTablePainter};
use crate::controller::TableController;
use crate::layout::AxisPart;
use crate::painter::TablePainter;

pub struct Table<T, P: TablePolicy<T>> {
    pub(crate) layout: Rc<RefCell<TableLayout>>,
    pub(crate) policy: P,
    pub(crate) lines: Vec<Box<dyn TableLine<T>>>,
    pub(crate) controller: Box<dyn TableController<T>>,
    pub(crate) painter: Box<dyn TablePainter<T>>,
}


pub struct TableContent<'a, T> {
    lines: &'a mut [Box<dyn TableLine<T>>],
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
        self.with_custom_line(WidgetTableLine::new(outer_lens, inner_lens, widget))
    }

    pub fn with_custom_line<L: TableLine<T> + 'static>(mut self, line: L) -> Self {
        self.add_line(line);
        self
    }

    pub(crate) fn add_line<L: TableLine<T> + 'static>(&mut self, line: L) {
        self.lines.push(Box::new(line));
        self.layout_mut().lines_mut().add_part(AxisPart::new(None));
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
            lines: vec![],
            controller: Box::new(DefaultTableController),
            painter: Box::new(DefaultTablePainter),
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
        let layout = self.layout.deref().borrow();
        let mut content = TableContent {lines: &mut self.lines};

        self.controller.event(ctx, event, data, env, &mut content, &layout);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for line in &mut self.lines {
            line.lifecycle(ctx, event, data, env);
        }
        self.controller.lifecycle(ctx, event, data, env);

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

        self.controller.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let mut layout = RefCell::borrow_mut(&self.layout);
        layout.prepare_layout(bc.max());

        for (index, line) in self.lines.iter_mut().enumerate() {
            line.layout(ctx, bc, data, env, &mut *layout, index);
        }
        for (index, line) in self.lines.iter_mut().enumerate() {
            line.arrange(ctx, data, env, &layout, index);
        }

        layout.table_size()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut layout = self.layout.deref().borrow();
        let mut content = TableContent {lines: &mut self.lines};

        self.painter.paint(ctx, data, env, &mut content, &layout);
    }
}

impl<'a, T: Data> TableContent<'a, T> {
    pub fn paint_background(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env, layout: &TableLayout) {
        for (index, line) in self.lines.iter_mut().enumerate() {
            line.paint(ctx, data, env, layout, index);
        }
    }

    pub fn paint_foreground(&mut self, _: &mut PaintCtx, _: &T, _: &Env, _: &TableLayout) {
        //TODO: paint hovered cells
    }

    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for line in self.lines.iter_mut() {
            line.event(ctx, event, data, env);
        }
    }
}