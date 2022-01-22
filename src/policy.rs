use druid::widget::ListIter;
use druid::Data;
use crate::{LineHeader, TableLine, TableLayout};

pub trait TablePolicy<T> {
    fn update(&mut self, old_data: &T, data: &T, widgets: &mut Vec<Box<dyn TableLine<T>>>, meta: &mut TableLayout);
}

pub trait HeaderTablePolicy<T>: TablePolicy<T> {
    fn update_headers(&mut self, old_data: &T, data: &T, widgets: &mut LineHeader<T, Self>, meta: &mut TableLayout) where Self: Sized;
}

pub struct Static;

impl<T: Data> TablePolicy<T> for Static {
    fn update(&mut self, _: &T, _: &T, _: &mut Vec<Box<dyn TableLine<T>>>, _: &mut TableLayout) {}
}

impl<T: Data> HeaderTablePolicy<T> for Static {
    fn update_headers(&mut self, _: &T, _: &T, _: &mut LineHeader<T, Self>, _: &mut TableLayout) {}
}