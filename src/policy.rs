use druid::Data;
use crate::{TableLine, TableLayout};

pub trait TablePolicy<T> {
    fn update(&mut self, old_data: &T, data: &T, widgets: &mut Vec<Box<dyn TableLine<T>>>, meta: &mut TableLayout);
}

pub struct Static;

impl<T: Data> TablePolicy<T> for Static {
    fn update(&mut self, _: &T, _: &T, _: &mut Vec<Box<dyn TableLine<T>>>, _: &mut TableLayout) {}
}