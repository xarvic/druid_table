use druid::{BoxConstraints, Rect, Size};
use druid::widget::Axis;

pub struct TableLayout {
    line_layout: Vec<AxisLayout>,
    element_layout: Vec<AxisLayout>,
    line_axis: Axis,
}

impl TableLayout {
    pub fn new(line_axis: Axis) -> Self {
        Self {
            line_layout: vec![],
            element_layout: vec![],
            line_axis,
        }
    }

    pub fn prepare_layout(&mut self) {
        for axis in &mut self.element_layout {
            if !axis.is_fixed {
                axis.size = 0.0;
            }
        }
        for axis in &mut self.line_layout {
            if !axis.is_fixed {
                axis.size = 0.0;
            }
        }
    }

    pub fn layout_rect(&self, line: usize, element: usize) -> Rect {
        let mut line_iter = self.line_layout.iter();

        let l1 = (&mut line_iter).take(line).map(AxisLayout::size).sum();
        let l2 = l1 + line_iter.next().unwrap().size();

        let mut element_iter = self.element_layout.iter();

        let e1 = (&mut element_iter).take(element).map(AxisLayout::size).sum();
        let e2 = e1 + element_iter.next().unwrap().size();

        Rect::from_points(self.line_axis.pack(e1, l1), self.line_axis.pack(e2, l2))
    }

    pub fn layout(&mut self, line: usize, element: usize, layout: impl FnOnce(&BoxConstraints) -> Size) {
        let line = &mut self.line_layout[line];
        let element = &mut self.element_layout[element];


        let inner_bc = BoxConstraints::new(
            Size::from(self.line_axis.pack(element.min, line.min)),
            Size::from(self.line_axis.pack(element.max, line.max)),
        );

        let size = layout(&inner_bc);

        if !element.is_fixed {
            element.size = element.size.max(self.line_axis.major(size));
        }
        if !line.is_fixed {
            line.size = line.size.max(self.line_axis.minor(size));
        }
    }

    pub fn table_size(&self) -> Size {
        Size::from(self.line_axis.pack(
            self.element_layout.iter().map(AxisLayout::size).sum(),
            self.line_layout.iter().map(AxisLayout::size).sum(),
        ))
    }

    pub(crate) fn add_line(&mut self, size: Option<f64>) {
        self.line_layout.push(AxisLayout::new(size));
    }

    pub fn element_count(&self) -> usize {
        self.element_layout.len()
    }

    pub fn set_element_count(&mut self, count: usize, size: Option<f64>) {
        if self.element_layout.len() > count {
            self.element_layout.truncate(count);
        } else {
            self.element_layout.reserve(count - self.element_layout.len());
            while self.element_layout.len() < count {
                self.element_layout.push(AxisLayout::new(size));
            }
        }
    }

    pub fn line_axis(&self) -> Axis {
        self.line_axis
    }
}

struct AxisLayout {
    size: f64,
    min: f64,
    max: f64,
    is_fixed: bool,
}

impl AxisLayout {
    pub fn new(size: Option<f64>) -> Self {
        AxisLayout {
            size: 0.0,
            min: size.unwrap_or(0.0),
            max: size.unwrap_or(f64::INFINITY),
            is_fixed: size.is_some(),
        }
    }

    pub fn size(&self) -> f64 {
        self.size
    }
}