use std::cmp::max;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Point, UpdateCtx, Widget, WidgetPod, Data, Size};
use druid::lens::Identity;
use druid::widget::ListIter;
use crate::TableLayout;

pub trait TableLine<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env);

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env);

    fn update(&mut self, ctx: &mut UpdateCtx, data: &T, env: &Env);

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env, meta: &mut TableLayout, line_index: usize);

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env, meta: &mut TableLayout, line_index: usize);

    fn arrange(&mut self, ctx: &mut LayoutCtx, data: &T, env: &Env, meta: &mut TableLayout, line_index: usize);

    fn element_count(&self, data: &T) -> usize;
}

impl<T: Data> TableLine<T> for Box<dyn TableLine<T>> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.deref_mut().event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.deref_mut().lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, data: &T, env: &Env) {
        self.deref_mut().update(ctx, data, env);
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env, meta: &mut TableLayout, line_index: usize) {
        self.deref_mut().paint(ctx, data, env, meta, line_index);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env, meta: &mut TableLayout, line_index: usize) {
        self.deref_mut().layout(ctx, bc, data, env, meta, line_index);
    }

    fn arrange(&mut self, ctx: &mut LayoutCtx, data: &T, env: &Env, meta: &mut TableLayout, line_index: usize) {
        self.deref_mut().arrange(ctx, data, env, meta, line_index);
    }

    fn element_count(&self, data: &T) -> usize {
        self.deref().element_count(data)
    }
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

    fn update_widget_count(&mut self, data: &S) -> bool {
        let Self {outer_lens, inner_lens, widgets, generate, ..} = self;
        outer_lens.with(data, |data| {
            if widgets.len() > data.data_len() {
                println!("remove {} widgets", widgets.len() - data.data_len());
                widgets.truncate(data.data_len());
                true
            } else if widgets.len() < data.data_len() {
                println!("add {} widgets", data.data_len() - widgets.len());
                widgets.extend(std::iter::repeat_with(generate).take(data.data_len() - widgets.len()));
                true
            } else {
                false
            }

        })
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
        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            if let Some(widget) = widgets.get_mut(index) {
                widget.update(ctx, data, env);
            }
        })));

        if self.update_widget_count(data) {
            ctx.children_changed();
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &S, env: &Env, _: &mut TableLayout, _: usize) {
        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            widgets[index].paint(ctx, data, env);
        })));
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &S, env: &Env, meta: &mut TableLayout, line_index: usize) {
        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            meta.layout(line_index, index, |bc|{
                widgets[index].layout(ctx, bc, data, env)
            });

            //TODO: align to baseline
        })));
    }

    fn arrange(&mut self, ctx: &mut LayoutCtx, data: &S, env: &Env, meta: &mut TableLayout, line_index: usize) {
        let Self {outer_lens, inner_lens, widgets, ..} = self;
        outer_lens.with(data, |data|data.for_each(|data, index|inner_lens.with(data, |data|{
            widgets[index].set_origin(ctx, data, env, meta.layout_rect(line_index, index).origin());

            //TODO: set paint insets
        })));
    }

    fn element_count(&self, data: &S) -> usize {
        self.outer_lens.with(data, |data|data.data_len())
    }
}

