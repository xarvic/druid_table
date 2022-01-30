use druid::{BoxConstraints, Rect, Size};
use druid::widget::Axis;

pub struct TableLayout {
    element_layout: AxisLayout,
    line_layout: AxisLayout,
    line_axis: Axis,
}

#[derive(Clone)]
pub struct AxisLayout {
    layout: Vec<AxisPart>,
    max_additional_size: f64,
}

#[derive(Clone, Copy)]
pub struct AxisPart {
    size: f64,
    min: f64,
    max: f64,
    is_fixed: bool,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum TableAxis {
    LineAxis,
    ElementAxis,
}

impl TableLayout {
    pub fn new(line_axis: Axis) -> Self {
        Self {
            element_layout: AxisLayout::new(),
            line_layout: AxisLayout::new(),
            line_axis,
        }
    }

    pub fn prepare_layout(&mut self, max_size: Size) {
        self.line_layout.prepare_layout(self.line_axis.minor(max_size));
        self.element_layout.prepare_layout(self.line_axis.major(max_size));
    }

    pub fn layout_rect(&self, line: usize, element: usize) -> Rect {
        let (l1, l2) = self.line_layout.current_layout(line);
        let (e1, e2) = self.element_layout.current_layout(element);

        Rect::from_points(self.line_axis.pack(e1, l1), self.line_axis.pack(e2, l2))
    }

    pub fn layout(&mut self, line: usize, element: usize, layout: impl FnOnce(&BoxConstraints) -> Size) {
        let line_constrains = self.line_layout.constrains(line);
        let element_constrains = self.element_layout.constrains(element);

        let inner_bc = BoxConstraints::new(
            Size::from(self.line_axis.pack(element_constrains.0, line_constrains.0)),
            Size::from(self.line_axis.pack(element_constrains.1, line_constrains.1)),
        );

        let size = layout(&inner_bc);

        self.line_layout.set_size(line, self.line_axis.minor(size));
        self.element_layout.set_size(element, self.line_axis.major(size));
    }

    pub fn table_size(&self) -> Size {
        Size::from(self.line_axis.pack(
            self.element_layout.size(),
            self.line_layout.size(),
        ))
    }

    pub fn line_axis(&self) -> Axis {
        self.line_axis
    }

    pub fn axis_direction(&self, table_axis: TableAxis) -> Axis {
        match table_axis {
            TableAxis::LineAxis => {self.line_axis}
            TableAxis::ElementAxis => {self.line_axis.cross()}
        }
    }

    pub fn elements(&self) -> &AxisLayout {
        &self.element_layout
    }

    pub fn lines(&self) -> &AxisLayout {
        &self.line_layout
    }

    pub fn table_axis(&self, table_axis: TableAxis) -> &AxisLayout {
        match table_axis {
            TableAxis::LineAxis => {&self.line_layout}
            TableAxis::ElementAxis => {&self.element_layout}
        }
    }

    pub fn elements_mut(&mut self) -> &mut AxisLayout {
        &mut self.element_layout
    }

    pub fn lines_mut(&mut self) -> &mut AxisLayout {
        &mut self.line_layout
    }

    pub fn table_axis_mut(&mut self, table_axis: TableAxis) -> &mut AxisLayout {
        match table_axis {
            TableAxis::LineAxis => {&mut self.line_layout}
            TableAxis::ElementAxis => {&mut self.element_layout}
        }
    }
}

impl AxisLayout {
    pub fn new() -> Self {
        Self {
            layout: vec![],
            max_additional_size: 0.0
        }
    }

    pub fn prepare_layout(&mut self, max_size: f64) {
        self.max_additional_size = max_size;
        for part in self.layout.iter_mut() {
            if !part.is_fixed {
                part.size = part.min;
            }
            self.max_additional_size -= part.size;
        }
    }

    pub fn constrains(&self, index: usize) -> (f64, f64) {
        self.layout[index].constrains(self.max_additional_size)
    }

    pub fn set_size(&mut self, index: usize, size: f64) {
        self.max_additional_size = self.layout[index].calc_space(size, self.max_additional_size);
    }

    pub fn current_layout(&self, index: usize) -> (f64, f64) {
        let mut iter = self.layout.iter();

        let l1 = (&mut iter).take(index).map(AxisPart::size).sum();
        (l1, l1 + iter.next().unwrap().size())
    }

    pub fn set_length(&mut self, length: usize, new: AxisPart) {
        if self.layout.len() > length {
            self.layout.truncate(length);
        } else {
            self.layout.reserve(length - self.layout.len());
            while self.layout.len() < length {
                self.layout.push(new);
            }
        }
    }

    pub fn add_part(&mut self, part: AxisPart) {
        self.layout.push(part);
    }

    pub fn length(&self) -> usize {
        self.layout.len()
    }

    pub fn size(&self) -> f64 {
        self.layout.iter().map(AxisPart::size).sum()
    }
}

impl AxisPart {
    pub fn new(size: Option<f64>) -> Self {
        AxisPart {
            size: 0.0,
            min: size.unwrap_or(0.0),
            max: size.unwrap_or(f64::INFINITY),
            is_fixed: size.is_some(),
        }
    }

    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn constrains(&self, max_additional_size: f64) -> (f64, f64) {
        let max = self.max.min(self.size + max_additional_size);
        (self.min, max)
    }

    pub fn calc_space(&mut self, size: f64, max_additional_space: f64) -> f64 {
        if !self.is_fixed {
            let old_size = self.size;
            self.size = self.size.max(size);
            (max_additional_space - self.size + old_size).max(0.0)
        } else {
            max_additional_space
        }
    }
}