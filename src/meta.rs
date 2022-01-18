use druid::widget::Axis;

pub struct TableMeta {
    pub line_sizes: Vec<f64>,
    pub line_size_fixed: bool,
    pub line_element_sizes: Vec<f64>,
    pub line_element_size_fixed: bool,
    pub line_axis: Axis,
}