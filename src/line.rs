use std::cmp::max;
use std::marker::PhantomData;
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx, UpdateCtx, Widget, WidgetPod};
use druid::lens::Identity;
use druid::widget::ListIter;
use crate::TableMeta;

pub trait TableLine<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env);

    fn lifecycle(&mut self, ctx: &LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env);

    fn update(&mut self, ctx: &UpdateCtx, data: &T, env: &Env);

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
    generate: Box<dyn Fn() -> W>,
    phantom: PhantomData<(S, T, U)>,
}

impl<
    S: Data,
    T: ListIter<U> + Data,
    U: Data,
    V: Data,

    L1: Lens<S, T>,
    L2: Lens<U, V>,
    W: Widget<V>,
> WidgetTableLine<S, T, U, V, L1, L2, W> {
    pub fn new(outer_lens: L1, inner_lens: L2, generate: impl Fn() -> W) -> Self {
        Self {
            outer_lens,
            inner_lens,
            widgets: vec![],
            generate: Box::new(generate),
            phantom: Default::default()
        }
    }
}

impl<
    S: Data,
    T: ListIter<U> + Data,
    U: Data,
    V: Data,

    L1: Lens<S, T>,
    L2: Lens<U, V>,
    W: Widget<V>,
> TableLine<S> for WidgetTableLine<S, T, U, V, L1, L2, W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut S, env: &Env) {
        self.outer_lens.with_mut(data, |data|data.for_each_mut(|data, index|self.lens.with_mut(data, |data|{
            self.widgets[index].event(ctx, event, data, env);
        })));
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &S, env: &Env) {
        self.outer_lens.with(data, |data|data.for_each(|data, index|self.lens.with(data, |data|{
            self.widgets[index].lifecycle(ctx, event, data, env);
        })));
    }

    fn update(&mut self, ctx: &mut UpdateCtx, data: &S, env: &Env) {
        if self.widgets.len() > data.data_len() {
            self.widgets.truncate(data.data_len());
        } else if self.widgets.len() < data.data_len() {
            self.widgets.extend(std::iter::repeat_with(&self.generate).take(data.data_len() - self.widgets.len()))
        }

        self.outer_lens.with(data, |data|data.for_each(|data, index|self.lens.with(data, |data|{
            self.widgets[index].update(ctx, data, env);
        })));
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &S, env: &Env, _: &mut TableMeta, _: usize) {
        self.outer_lens.with(data, |data|data.for_each(|data, index|self.lens.with(data, |data|{
            self.widgets[index].paint(ctx, data, env);
        })));
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &S, env: &Env, meta: &mut TableMeta, line_index: usize) {
        let mut current_line_length = 0.0;
        self.outer_lens.with(data, |data|data.for_each(|data, index|self.lens.with(data, |data|{
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

            let size = self.widgets[index].layout(ctx, &inner_bc, inner, env);

            if !meta.line_size_fixed {
                meta.line_sizes[line_index] = meta.line_axis.minor(size);
            }

            if !meta.line_element_size_fixed {
                meta.line_element_sizes[index] = meta.line_axis.major(size);
            }

            //TODO: align to baseline
        })));
    }

    fn arrange(&mut self, ctx: &mut LayoutCtx, data: &S, env: &Env, meta: &TableMeta, line_index: usize) {
        self.outer_lens.with(data, |data|data.for_each(|data, index|self.lens.with(data, |data|{
            self.widgets[index].set_origin(ctx, data, env, Pont::from(meta.line_axis.pack(
                meta.line_element_sizes[index],
                meta.line_sizes[line_index],
            )));

            //TODO: set paint insets
        }));
    }
}

