use std::cell::{RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, Size, UpdateCtx, Vec2, Widget, WidgetPod, Data, RenderContext, Affine};
use druid::commands::SCROLL_TO_VIEW;
use druid::scroll_component::ScrollComponent;
use druid::widget::{Axis, ClipBox, ListIter, Scroll, Viewport};
use crate::{AxisPart, Static, Table, TableAxis, TableLayout, TableLine, TablePolicy, WidgetTableLine};
use crate::util::set_len;

pub struct HeaderTable<T: Data, P: TablePolicy<T>> {
    table: WidgetPod<T, ClipBox<T, Table<T, P>>>,

    line_header: WidgetPod<T, ClipBox<T, Header<T>>>,
    line_header_width: f64,
    element_header: Option<WidgetPod<T, ClipBox<T, Header<T>>>>,
    element_header_width: f64,

    scroll_component: ScrollComponent,
}

type HeaderBuilder<T> = Box<dyn FnMut(&T, &T, usize, &mut Vec<HeaderWidget<T>>, &mut UpdateCtx)>;

pub type HeaderWidget<T> = WidgetPod<HeaderData<T>, Box<dyn Widget<HeaderData<T>>>>;

struct Header<T> {
    widgets: Vec<HeaderWidget<T>>,
    builder: HeaderBuilder<T>,
    layout: Rc<RefCell<TableLayout>>,
    table_axis: TableAxis,
}

#[derive(Copy, Clone, Data)]
pub struct HeaderData<T> {
    pub(crate) data: T,
    pub(crate) index: usize,
    pub(crate) part: AxisPart,
}

impl<T: Data, P: TablePolicy<T>> HeaderTable<T, P> {
    pub fn new_dynamic(axis: Axis, policy: P, line_headers: HeaderBuilder<T>, line_header_width: f64) -> Self {
        let layout = Rc::new(RefCell::new(TableLayout::new(axis)));

        Self {
            table: WidgetPod::new(
                ClipBox::new(Table::new(policy, layout.clone()))
                    .constrain_vertical(true)
                    .constrain_horizontal(true)
                ),
            line_header: WidgetPod::new(
                ClipBox::new(Header::new(line_headers, layout, TableAxis::LineAxis))
                    .constrain_vertical(true)
                    .constrain_horizontal(true)
            ),
            line_header_width,
            element_header: None,
            element_header_width: 0.0,
            scroll_component: ScrollComponent::new(),
        }
    }

    pub fn with_element_header(mut self, builder: impl Fn() -> Box<dyn Widget<HeaderData<T>>> + 'static, element_header_width: f64) -> Self {
        self.element_header = Some(WidgetPod::new(ClipBox::new(Header::new(
            Box::new(move|_, _, length, list, ctx| {
                if list.len() != length {
                    ctx.children_changed();
                    set_len(list, length, ||WidgetPod::new(builder()));
                }
            }),
            self.table.widget().child().layout.clone(),
            TableAxis::LineAxis
        ))
            .constrain_horizontal(true)
            .constrain_vertical(true)
        ));
        self.element_header_width = element_header_width;
        self
    }

    fn adjust_scrolling(&mut self, table_axis: Axis) {
        let scroll_offset = self.table.widget().viewport_origin();

        let line_offset = table_axis.minor_pos(scroll_offset);
        self.line_header.widget_mut().pan_to(Point::from(table_axis.pack(0.0, line_offset)));

        let element_offset = table_axis.major_pos(scroll_offset);
        if let Some(element_header) = &mut self.element_header {
            element_header.widget_mut().pan_to(Point::from(table_axis.pack(element_offset, 0.0)));
        }
    }

    fn with_port(&mut self, update: impl FnOnce(&mut Viewport, &mut ScrollComponent)) {
        let mut changed = false;
        let Self {table, scroll_component, ..} = self;
        table.widget_mut().with_port(|viewport|{
            let old_vieworigin = viewport.view_origin;
            update(viewport, scroll_component);
            if old_vieworigin != viewport.view_origin {
                changed = true;
            }
        });

        if changed {
            let axis = self.table_layout().line_axis();
            self.adjust_scrolling(axis);
        }
    }

    fn table_layout(&self) -> RefMut<TableLayout> {
        self.table.widget().child().layout_mut()
    }
}

impl<T: Data> HeaderTable<T, Static> {
    pub fn new_static(axis: Axis, line_header_width: f64) -> Self {
        Self::new_dynamic(axis, Static, Box::new(|_, _, _, _, _|()), line_header_width)
    }

    pub fn with_custom_line<L: TableLine<T> + 'static>(mut self, line: L, header: impl Widget<HeaderData<T>> + 'static) -> Self {
        self.table.widget_mut().child_mut().add_line(line);
        self.line_header.widget_mut().child_mut().widgets.push(WidgetPod::new(Box::new(header)));
        self
    }

    pub fn with_line<
        T2: ListIter<U> + Data,
        U: Data,
        V: Data,

        L1: Lens<T, T2> + 'static,
        L2: Lens<U, V> + 'static,
        W: Widget<V> + 'static,
        F: Fn() -> W + 'static,

    >(mut self, outer_lens: L1, inner_lens: L2, widget: F, header: impl Widget<HeaderData<T>> + 'static) -> Self {
        self.with_custom_line(WidgetTableLine::new(outer_lens, inner_lens, widget), header)
    }
}

impl<T: Data, P: TablePolicy<T>> Widget<T> for HeaderTable<T, P> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.with_port(|port, scroll_component| {
            scroll_component.event(port, ctx, event, env);
        });

        if !ctx.is_handled() {
            self.table.event(ctx, event, data, env);
            self.line_header.event(ctx, event, data, env);
            if let Some(element_header) = &mut self.element_header {
                element_header.event(ctx, event, data, env);
            }
        }

        // Handle scroll after the inner widget processed the events, to prefer inner widgets while
        // scrolling.
        self.with_port(|port, scroll_component| {
            scroll_component.handle_scroll(port, ctx, event, env);
        });

        if !self.scroll_component.are_bars_held() {
            // We only scroll to the component if the user is not trying to move the scrollbar.
            if let Event::Notification(notification) = event {
                if let Some(& (mut global_highlight_rect)) = notification.get(SCROLL_TO_VIEW) {
                    ctx.set_handled();

                    //TODO: fix

                    if notification.route() != self.table.id() {
                        let clipped_axis = if notification.route() == self.line_header.id() {
                            self.table_layout().line_axis()
                        } else {
                            self.table_layout().line_axis().cross()
                        };
                        let clipped_span = clipped_axis.major_span(global_highlight_rect);
                        global_highlight_rect = Rect::from_points(
                            clipped_axis.pack(clipped_span.0, 0.0),
                            clipped_axis.pack(clipped_span.1, 0.0),
                        );
                    };

                    let view_port_changed = self.table.widget_mut().default_scroll_to_view_handling(ctx, global_highlight_rect);

                    if view_port_changed {
                        self.scroll_component
                            .reset_scrollbar_fade(|duration| ctx.request_timer(duration), env);
                    }
                }
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.scroll_component.lifecycle(ctx, event, env);
        self.table.lifecycle(ctx, event, data, env);
        self.line_header.lifecycle(ctx, event, data, env);
        if let Some(element_header) = &mut self.element_header {
            element_header.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.table.update(ctx, data, env);
        self.line_header.update(ctx, data, env);
        if let Some(element_header) = &mut self.element_header {
            element_header.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        //TODO: fix layout
        // The table should do layout first to set the sizes of the cells for the header
        // but the headers should do layout first to determine how much space is left for the scroll.
        // for now we use a fixed size for the headers

        let table_axis = self.table_layout().line_axis();
        let header_space = Size::from(table_axis.pack(self.line_header_width, self.element_header_width));
        let table_bc = bc.shrink(header_space);

        //Layout Table
        let old_table_size = self.table.layout_rect().size();
        let offset = self.table.widget().viewport_origin();
        let cell_offset = self.table_layout().as_cell_offset(offset.to_vec2());

        let table_size = self.table.layout(ctx, &table_bc, data, env);
        self.table.set_origin(ctx, data, env, header_space.to_vec2().to_point());

        // Adjust Scroll to point to the same cell.
        // This is important for virtual scrolling.
        //TODO: allow sticky borders
        let offset = self.table_layout().from_cell_offset(cell_offset);
        self.with_port(|port, _|{port.pan_to(offset.to_point());});

        if old_table_size != table_size {
            self.scroll_component.reset_scrollbar_fade(|duration|ctx.request_timer(duration), env);
        }

        //Layout Headers
        self.line_header.layout(ctx, &BoxConstraints::tight(Size::from(table_axis.pack(self.line_header_width, table_axis.minor(table_size)))), data, env);
        self.line_header.set_origin(ctx, data, env, Point::from(table_axis.pack(0.0, self.element_header_width)));
        if let Some(element_header) = &mut self.element_header {
            element_header.layout(ctx, &BoxConstraints::tight(Size::from(table_axis.pack(self.element_header_width, table_axis.minor(table_size)))), data, env);
            element_header.set_origin(ctx, data, env, Point::from(table_axis.pack(self.line_header_width, 0.0)));
        }

        table_size + header_space
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.table.paint(ctx, data, env);
        ctx.with_save(|ctx|{
            ctx.transform(Affine::translate(self.table.layout_rect().origin().to_vec2()));
            self.scroll_component.draw_bars(ctx, &self.table.widget().viewport(), env);
        });
        self.line_header.paint(ctx, data, env);
        if let Some(element_header) = &mut self.element_header {
            element_header.paint(ctx, data, env);
        }
    }
}

impl<T: Data> Header<T> {
    fn new(builder: HeaderBuilder<T>, layout: Rc<RefCell<TableLayout>>, table_axis: TableAxis) -> Self {
        Self {
            widgets: vec![],
            builder,
            layout,
            table_axis
        }
    }

    fn for_each(&mut self, data: &T, mut f: impl FnMut(&HeaderData<T>, &mut HeaderWidget<T>)) {
        let table_layout = self.layout.deref().borrow();
        let layout = table_layout.table_axis(self.table_axis);

        for (index, widget) in self.widgets.iter_mut().enumerate() {
            let header_data = HeaderData {
                data: data.to_owned(),
                index,
                part: layout.get(index)
            };
            f(&header_data, widget);
        }
    }
}

impl<T: Data> Widget<T> for Header<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let mut table_layout = self.layout.deref().borrow_mut();
        let layout = table_layout.table_axis_mut(self.table_axis);

        for (index, widget) in self.widgets.iter_mut().enumerate() {
            let part = layout.get(index);
            let mut header_data = HeaderData {
                data: data.to_owned(),
                index,
                part,
            };
            widget.event(ctx, event, &mut header_data, env);
            *data = header_data.data;
            if !part.same(&header_data.part) {
                layout.set(index, part);
                ctx.request_layout();
                //TODO: notify the table
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.for_each(data, |data, widget|widget.lifecycle(ctx, event, data, env));
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.for_each(data, |data, widget|widget.update(ctx, data, env));

        let length = self.layout.deref().borrow().table_axis(self.table_axis).length();

        (self.builder)(old_data, data, length, &mut self.widgets, ctx);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let axis = self.layout.deref().borrow().header_direction(self.table_axis);
        let max_cross = axis.minor(bc.max());

        let mut cross_width: f64 = 0.0;
        let mut advance = 0.0;

        self.for_each(data, |data, widget|{
            let inner_bc = BoxConstraints::new(
                Size::from(axis.pack(data.part.size(), 0.0)),
                Size::from(axis.pack(data.part.size(), max_cross)),
            );
            let size = widget.layout(ctx, &inner_bc, data, env);
            widget.set_origin(ctx, data, env, dbg!(Point::from(axis.pack(advance, 0.0))));

            cross_width = cross_width.max(axis.minor(size));
            advance += data.part.size();
        });

        Size::from(axis.pack(advance, cross_width))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.for_each(data, |data, widget|widget.paint(ctx, data, env));
    }
}