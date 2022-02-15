use std::cell::{RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, Size, UpdateCtx, Widget, WidgetPod, Data};
use druid::widget::{Axis, ClipBox, ListIter, Scroll};
use crate::{AxisPart, Static, Table, TableAxis, TableLayout, TableLine, TablePolicy, WidgetTableLine};
use crate::util::set_len;

pub struct HeaderTable<T: Data, P: TablePolicy<T>> {
    table: WidgetPod<T, Scroll<T, Table<T, P>>>,

    line_header: WidgetPod<T, ClipBox<T, Header<T>>>,
    line_header_width: f64,
    element_header: Option<WidgetPod<T, ClipBox<T, Header<T>>>>,
    element_header_width: f64,

    last_view_origin: Point,
}

type HeaderBuilder<T> = Box<dyn FnMut(&T, &T, usize, &mut Vec<HeaderWidget<T>>)>;

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
            table: WidgetPod::new(Scroll::new(Table::new(policy, layout.clone()))),
            line_header: WidgetPod::new(
                ClipBox::new(Header::new(line_headers, layout, TableAxis::LineAxis))
                    .constrain_vertical(true)
                    .constrain_horizontal(true)
            ),
            line_header_width,
            element_header: None,
            element_header_width: 0.0,
            last_view_origin: Point::ORIGIN,
        }
    }

    pub fn with_element_header(mut self, builder: impl Fn() -> Box<dyn Widget<HeaderData<T>>> + 'static, element_header_width: f64) -> Self {
        self.element_header = Some(WidgetPod::new(ClipBox::new(Header::new(
            Box::new(move|_, _, length, list| {
                set_len(list, length, ||WidgetPod::new(builder()));
            }),
            self.table.widget().child().layout.clone(),
            TableAxis::ElementAxis
        ))
            .constrain_horizontal(true)
            .constrain_vertical(true)
        ));
        self.element_header_width = element_header_width;
        self
    }

    fn adjust_scrolling(&mut self) {
        let table_axis = self.table_layout().line_axis();

        let line_offset = table_axis.minor_pos(self.last_view_origin);
        self.line_header.widget_mut().pan_to(Point::from(table_axis.pack(0.0, line_offset)));

        let element_offset = table_axis.major_pos(self.last_view_origin);
        if let Some(element_header) = &mut self.element_header {
            element_header.widget_mut().pan_to(Point::from(table_axis.pack(element_offset, 0.0)));
        }
    }

    fn table_layout(&self) -> RefMut<TableLayout> {
        self.table.widget().child().layout_mut()
    }
}

impl<T: Data> HeaderTable<T, Static> {
    pub fn new_static(axis: Axis, line_header_width: f64) -> Self {
        Self::new_dynamic(axis, Static, Box::new(|_, _, _, _|()), line_header_width)
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

    >(self, outer_lens: L1, inner_lens: L2, widget: F, header: impl Widget<HeaderData<T>> + 'static) -> Self {
        self.with_custom_line(WidgetTableLine::new(outer_lens, inner_lens, widget), header)
    }
}

impl<T: Data, P: TablePolicy<T>> Widget<T> for HeaderTable<T, P> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.table.event(ctx, event, data, env);
        self.line_header.event(ctx, event, data, env);
        if let Some(element_header) = &mut self.element_header {
            element_header.event(ctx, event, data, env);
        }

        //TODO: handle SCROLL_TO_VIEW from headers

        //Sync headers with content
        let new_view_origin = self.table.widget().offset().to_point();
        if new_view_origin != self.last_view_origin {
            self.last_view_origin = new_view_origin;
            self.adjust_scrolling();
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.table.lifecycle(ctx, event, data, env);
        self.line_header.lifecycle(ctx, event, data, env);
        if let Some(element_header) = &mut self.element_header {
            element_header.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
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
        let offset = self.table.widget().offset();
        let cell_offset = self.table_layout().as_cell_offset(offset);

        let table_size = self.table.layout(ctx, &table_bc, data, env);
        self.table.set_origin(ctx, data, env, header_space.to_vec2().to_point());

        // Adjust Scroll to point to the same cell.
        // This is important for virtual scrolling.
        //TODO: allow sticky borders
        let view_origin = self.table_layout().from_cell_offset(cell_offset).to_point();
        if view_origin != self.last_view_origin {
            self.last_view_origin = view_origin;

            // This is a hack until we can use ScrollComponent directly
            let view_rect = Rect::from_origin_size(view_origin, self.table.layout_rect().size());
            self.table.widget_mut().scroll_to(view_rect);
            self.adjust_scrolling();
        }

        //Layout Headers
        self.line_header.layout(ctx, &BoxConstraints::tight(Size::from(table_axis.pack(self.line_header_width, table_axis.minor(table_size)))), data, env);
        self.line_header.set_origin(ctx, data, env, Point::from(table_axis.pack(0.0, self.element_header_width)));
        if let Some(element_header) = &mut self.element_header {
            element_header.layout(ctx, &BoxConstraints::tight(Size::from(table_axis.pack(table_axis.major(table_size), self.element_header_width))), data, env);
            element_header.set_origin(ctx, data, env, Point::from(table_axis.pack(self.line_header_width, 0.0)));
        }

        table_size + header_space
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.table.paint(ctx, data, env);
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
        let length = self.layout.deref().borrow().table_axis(self.table_axis).length();
        (self.builder)(data, data, length, &mut self.widgets);

        self.for_each(data, |data, widget|widget.lifecycle(ctx, event, data, env));
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.for_each(data, |data, widget|widget.update(ctx, data, env));

        let length = self.layout.deref().borrow().table_axis(self.table_axis).length();
        if length != self.widgets.len() {
            (self.builder)(old_data, data, length, &mut self.widgets);
            ctx.children_changed();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let axis = self.layout.deref().borrow().header_direction(self.table_axis);
        let max_cross = axis.minor(bc.max());

        let mut cross_width: f64 = 0.0;
        let mut advance = self.layout.deref().borrow().table_axis(self.table_axis).start_padding();

        self.for_each(data, |data, widget|{
            let inner_bc = BoxConstraints::new(
                Size::from(axis.pack(data.part.size(), 0.0)),
                Size::from(axis.pack(data.part.size(), max_cross)),
            );
            let size = widget.layout(ctx, &inner_bc, data, env);
            widget.set_origin(ctx, data, env, Point::from(axis.pack(advance, 0.0)));

            cross_width = cross_width.max(axis.minor(size));
            advance += data.part.advance();
        });

        Size::from(axis.pack(advance, cross_width))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.for_each(data, |data, widget|widget.paint(ctx, data, env));
    }
}

impl<T: Data> HeaderData<T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn width(&self) -> f64 {
        self.part.size()
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}