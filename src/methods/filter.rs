use crate::points::{Point, Points};
use std::collections::HashMap;

pub type FilterFn = Box<dyn Fn(&Point) -> bool>;
pub type FilterProducer = Box<dyn Fn(&Points) -> FilterFn>;
pub const DEFAULT_KEY: &str = "default";

pub fn do_nothing() -> FilterProducer {
    Box::new(move |_points: &Points| Box::new(move |point: &Point| false))
}

pub fn upper_half() -> FilterProducer {
    Box::new(move |points: &Points| {
        let len = points.len() as f32;
        let sum: f32 = points
            .get_clone_data()
            .into_iter()
            .map(|point| point.point_coord.y)
            .sum();
        let mean = sum / len;

        Box::new(move |point: &Point| point.point_coord.y > mean)
    })
}

pub fn get_collection() -> HashMap<String, FilterProducer> {
    let mut filter_methods = HashMap::new();
    filter_methods.insert(DEFAULT_KEY.to_string(), do_nothing());
    filter_methods.insert("upper_half".to_string(), upper_half());
    filter_methods
}
