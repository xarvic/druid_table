use std::cmp::max;
use std::marker::PhantomData;
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Point, UpdateCtx, Widget, WidgetPod, Data, Size};
use druid::lens::Identity;
use druid::widget::ListIter;
use crate::TableMeta;

pub trait TableLine<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env);

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env);

    fn update(&mut self, ctx: &mut UpdateCtx, data: &T, env: &Env);

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env, meta: &mut TableMeta, line_index: usize);

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env, meta: &mut TableMeta, line_index: usize);

    fn arrange(&mut self, ctx: &mut LayoutCtx, data: &T, env: &Env, meta: &mut TableMeta, line_index: usize);
}

pub struct WidgetTableLine<
    S: Data,
    T: ListIter<U> + Data,
    U: Data,
    V: Data,

    L1: Lens<S, T>,
    L2: Lens<U, V>,
    W: Widget<V>,
> {
    outer_lens: L1,
    inner_lens: L2,
    widgets: Vec<WidgetPod<V, W>>,
    generate: Box<dyn Fn() -> WidgetPod<V, W>>,
    phantom: PhantomData<(S, T, U)>,
}

impl<
    S: Data,
    T: ListIter<U> + Data,
    U: Data,
    V: Data,

    L1: Lens<S, T>,
    L2: Lens<U, V>,
    W: Widget<V> + 'static,
> WidgetTableLine<S, T, U, V, L1, L2, W> {
    pub fn new(outer_lens: L1, inner_lens: L2, generate: impl Fn() -> W + 'static) -> Self {
        Self {
            outer_lens,
            inner_lens,
            widgets: vec![],
            generate: Box::new(move||WidgetPod::new(generate())),
            phantom: Default::default()
        }
    }

    fn update_widget_count(&mut self, data: &S) {
        let Self {outer_lens, inner_lens, widgets, generate, ..} = self;
        outer_lens.with(data, |data| {
            if widgets.len() > data.data_len() {
                println!("remove {} widgets", widgets.len() - data.data_len());
                widgets.truncate(data.data_len());
            } else if widgets.len() < data.data_len() {
                println!("add {} widgets", data.data_len() - widgets.len());
                widgets.extend(std::iter::repeat_with(generate).take(data.data_len() - widgets.len()));
            }
        });
    }
}

impl<
    S: Data,
    T: ListIter<U> + Data,
    U: Data,
    V: Data,

    L1: Lens<S, T>,
    L2: Lens<U, V>,
    W: Widget<V> + 'static,
> TableLine<S> for WidgetTableLine<S, T, U, V, L1, L2, W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut S, env: &Env) {
        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with_mut(data, |data|data.for_each_mut(|data, index|inner_lens.with_mut(data, |data|{
            widgets[index].event(ctx, event, data, env);
        })));
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &S, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.update_widget_count(data);
        }

        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            widgets[index].lifecycle(ctx, event, data, env);
        })));
    }

    fn update(&mut self, ctx: &mut UpdateCtx, data: &S, env: &Env) {
        self.update_widget_count(data);

        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            widgets[index].update(ctx, data, env);
        })));
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &S, env: &Env, _: &mut TableMeta, _: usize) {
        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            widgets[index].paint(ctx, data, env);
        })));
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &S, env: &Env, meta: &mut TableMeta, line_index: usize) {

        //TODO ensure all lines have the same length
        if meta.line_element_sizes.len() < self.widgets.len() {
            meta.line_element_sizes.extend(std::iter::repeat(0.0).take(self.widgets.len() - meta.line_element_sizes.len()));
        } else if meta.line_element_sizes.len() > self.widgets.len() {
            meta.line_element_sizes.truncate(self.widgets.len());
        }

        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            let (min_line, max_line) = if meta.line_size_fixed {
                (meta.line_sizes[line_index], meta.line_sizes[line_index])
            } else {
                (meta.line_axis.minor(bc.min()), meta.line_axis.minor(bc.min()))
            };
            let (min_element, max_element) = if meta.line_size_fixed {
                (meta.line_sizes[line_index], meta.line_sizes[line_index])
            } else {
                (meta.line_axis.minor(bc.min()), meta.line_axis.minor(bc.min()))
            };
            let inner_bc = BoxConstraints::new(
                Size::from(meta.line_axis.pack(min_element, min_line)),
                Size::from(meta.line_axis.pack(max_element, max_line)),
            );


            let size = widgets[index].layout(ctx, &inner_bc, data, env);

            if !meta.line_size_fixed {
                meta.line_sizes[line_index] = meta.line_axis.minor(size);
            }

            if !meta.line_element_size_fixed {
                meta.line_element_sizes[index] = meta.line_axis.major(size);
            }

            //TODO: align to baseline
        })));
    }

    fn arrange(&mut self, ctx: &mut LayoutCtx, data: &S, env: &Env, meta: &mut TableMeta, line_index: usize) {
        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            widgets[index].set_origin(ctx, data, env, Point::from(meta.line_axis.pack(
                meta.line_element_sizes[index],
                meta.line_sizes[line_index],
            )));

            //TODO: set paint insets
        })));
    }
}

