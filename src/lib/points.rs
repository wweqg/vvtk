use ply_rs::ply;
use kd_tree::{ KdPoint, KdTree };
use std::iter::Iterator;

use crate::color::{ Color, PointColor };
use crate::coordinate::{ Coordinate, PointCoordinate };
use crate::renderer;
use nalgebra::Point3;
use std::any::type_name;
use std::cmp::Ordering;

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[derive(Clone)]
pub struct Points {
    pub data: Vec<Point>,
    pub delta_pos_vector: Vec<Point3<f32>>,
    pub delta_colours: Vec<Point3<f32>>,
    pub reference_frame: Vec<Point>
}

impl Points {
    pub fn new() -> Self {
        Points {
            data: Vec::new(),
            delta_pos_vector: Vec::new(),
            delta_colours: Vec::new(),
            reference_frame: Vec::new()
        }
    }

    pub fn add(&mut self, elem: Point) {
        self.data.push(elem);
    }
    
    pub fn of(data: Vec<Point>) -> Self {
        
        Points {
            data: data,
            delta_pos_vector: Vec::new(),
            delta_colours: Vec::new(),
            reference_frame: Vec::new()

        }
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn get_data(self) -> Vec<Point> {
        self.data
    }

    pub fn get_colors(self) -> Color {
        Color::new(self.data.into_iter().map(|point| point.point_color).collect())
    }

    pub fn get_coords(self) -> Coordinate {
        Coordinate::new(self.data.into_iter().map(|point| point.point_coord).collect())
    }

    pub fn get_coords_cols(self) -> (Coordinate, Color) {
        let mut coords = Vec::new();
        let mut colors = Vec::new();
        for point in self.data {
            coords.push(point.point_coord);
            colors.push(point.point_color);
        }

        (Coordinate::new(coords), Color::new(colors))
    }

    pub fn render(&self) {
        let mut renderer = renderer::Renderer::new();
        while renderer.rendering() {
            renderer.render_frame(&self);
        }
    }

    pub fn to_kdtree(self) -> KdTree<Point>{
        KdTree::build_by_ordered_float(self.get_data())
    }

    pub fn mark_mapped_points(&mut self)
    {
        let mut mapped_points = 0;
        for point in self.reference_frame.iter_mut()
        {
            if point.mapping == 0
            {
                point.point_color = PointColor::new(0, 255, 0);
            }
            else
            {
                mapped_points += 1;
            }
        }

        println!("mapped points: {}; total points: {}", mapped_points, self.reference_frame.len());
    }

    pub fn average_points_recovery(&mut self, points: Points) -> (Points, Points) {
        self.reference_frame = points.clone().get_data();

        let kd_tree = points.to_kdtree();
        let x = self.clone();
        
        let point_data = Points::of(x.get_data().into_iter()
            .map(|point| point.get_average(&point.get_nearest(&kd_tree, &mut self.reference_frame)))
            .collect());      
        
        
        self.frame_delta(point_data.clone());
        
        self.mark_mapped_points();

        (point_data, Points::of(self.reference_frame.clone()))
    }

    pub fn closest_with_ratio_average_points_recovery(&mut self, points: Points, penalize_coor: f32, penalize_col: f32, penalize_mapped: f32) -> (Points, Points){
        self.reference_frame = points.clone().get_data();

        let kd_tree = points.to_kdtree();
        let x = self.clone();

        let point_data = Points::of(x.get_data().into_iter()
                    .map(|point| point.get_average_closest_from_kdtree(&kd_tree, penalize_coor, penalize_col, &mut self.reference_frame, penalize_mapped))
                    .collect());

        self.frame_delta(point_data.clone());
        // point_data

        self.mark_mapped_points();

        (point_data, Points::of(self.reference_frame.clone()))
    }

    //accepts argument of points in case this function is called in main before any interpolation function is called i.e. will be used to calculate a simple delta
    // this function is also called in each of the interpolation functions, taking in the vector of closest points i.e. fn can be used in 2 ways
    pub fn frame_delta(&mut self, prev: Points)
    {
        // let next_coordinates_obj = self.clone().get_coords();
        // let next_colours_obj = self.clone().get_colors();

        let (next_coordinates_obj, next_colours_obj)  = self.clone().get_coords_cols();

        let next_coordinates = next_coordinates_obj.getPointCoorVec();        
        let next_colours = next_colours_obj.getPointColVec();

        // let prev_coordinates_obj = prev.clone().get_coords();
        // let prev_colours_obj = prev.get_colors();

        let (prev_coordinates_obj, prev_colours_obj)  = prev.get_coords_cols();

        let prev_coordinates = prev_coordinates_obj.getPointCoorVec();        
        let prev_colours = prev_colours_obj.getPointColVec();

        for (pos, _e) in prev_coordinates.iter().enumerate()
        {
            let (x, y, z) = (next_coordinates[pos].x - prev_coordinates[pos].x, next_coordinates[pos].y - prev_coordinates[pos].y, next_coordinates[pos].z - prev_coordinates[pos].z);
            self.delta_pos_vector.push(Point3::new(x, y, z));
        }

        for (pos, _e) in prev_colours.iter().enumerate()
        {
            let (x, y, z) = (next_colours[pos].red as f32 - prev_colours[pos].red as f32, next_colours[pos].green as f32 - prev_colours[pos].green as f32, next_colours[pos].blue as f32 - prev_colours[pos].blue as f32);
            self.delta_colours.push(Point3::new(x , y , z ));
        }
    }

    pub fn get_delta_pos_vector(&self) ->  Vec<Point3<f32>>
    {
        self.delta_pos_vector.clone()
    }

    pub fn get_delta_colours(&self) ->  Vec<Point3<f32>>
    {
        self.delta_colours.clone()
    }
}

impl IntoIterator for Points {
    type Item = Point;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Point {
    point_coord: PointCoordinate,
    point_color: PointColor,
    mapping: u16,
    index: usize
}


impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.point_coord == other.point_coord &&
        self.point_color == other.point_color
    }
}


impl Point {
    pub fn new(point_coord: PointCoordinate, point_color: PointColor, mapping: u16, index: usize) -> Self {
        Point {
            point_coord: point_coord,
            point_color: point_color,
            mapping: mapping,
            index: index
        }
    }

    pub fn partial_cmp(&self, other: &Point) -> Option<Ordering>
    {
        let mut first_dist_from_ori = f32::powf(self.point_coord.x, 2.0) + f32::powf(self.point_coord.y, 2.0) + f32::powf(self.point_coord.z, 2.0);
        first_dist_from_ori = first_dist_from_ori.sqrt();

        let next_dist_from_ori = f32::powf(other.point_coord.x, 2.0) + f32::powf(other.point_coord.y, 2.0) + f32::powf(other.point_coord.z, 2.0);

        if first_dist_from_ori < next_dist_from_ori
        {
           return Some(Ordering::Less);
        }

        else if first_dist_from_ori > next_dist_from_ori
        {
            return Some(Ordering::Greater);
        }

        return Some(Ordering::Equal);
    }

    fn new_default() -> Self {
        Point {
            point_coord: PointCoordinate::new_default(),
            point_color: PointColor::new_default(),
            mapping: 0,
            index: 0
        }
    }

    pub fn get_coord(&self) -> &PointCoordinate {
        &self.point_coord
    }

    pub fn get_color(&self) -> &PointColor {
        &self.point_color
    }

    pub fn set_index(&mut self, idx: usize){
        self.index = idx;
    }

    pub fn get_index(&mut self) -> usize{
        self.index
    }

    pub fn get_nearest(&self, kd_tree: &KdTree<Point>, reference_frame: &mut Vec<Point>) -> Point {
        let mut nearest_point = kd_tree.nearest(self).unwrap().item.clone();
        reference_frame[nearest_point.get_index()].mapping += 1;

        nearest_point
    }

    //penalization
    //update count in kdtree point
    pub fn get_nearests(&self, kd_tree: &KdTree<Point>, quantity: usize) -> Points {
        Points::of(kd_tree.nearests(self, quantity).into_iter().map(|found| found.item.clone()).collect())
    }

    pub fn get_average(&self, another_point: &Point) -> Point {
        Point::new(self.clone().get_coord().get_average(another_point.get_coord()), 
                    self.clone().get_color().get_average(another_point.get_color()), 0, self.index)
    }

    fn get_coord_delta(&self, another_point: &Point) -> f32 {
        self.clone().get_coord().get_coord_delta(&another_point.clone().get_coord())
    }

    fn get_color_delta(&self, another_point: &Point) -> f32 {
        self.clone().get_color().get_color_delta(&another_point.clone().get_color())
    }

    ///penalization 
    fn get_difference(&self, another_point: &Point, penalize_coor: f32, penalize_col:f32, another_point_mapping: u16, penalize_mapped: f32) -> f32 {
        self.get_coord_delta(another_point) * penalize_coor  +
        self.get_color_delta(another_point) * penalize_col + 
        another_point_mapping as f32 * penalize_mapped

    }

    
    fn get_closest(&self, points: Points, penalize_coor: f32, penalize_col: f32, reference_frame: &mut Vec<Point>, penalize_mapped: f32) -> Point {
        let mut min: f32 = f32::MAX;
        let mut result: Point = Point::new_default();

        for mut point in points.data {
            let map = reference_frame[point.get_index()].mapping;
            let cur = self.get_difference(&point, penalize_coor, penalize_col, map, penalize_mapped);
            if cur < min {
                min = cur;
                reference_frame[point.get_index()].mapping += 1;
                result = point
            }
        }
        reference_frame[result.get_index()].mapping += 1;
        result
    }

    fn get_average_closest(&self, points: Points, penalize_coor:f32, penalize_col: f32, reference_frame: &mut Vec<Point>, penalize_mapped: f32) -> Point {
        self.get_average(&self.get_closest(points, penalize_coor, penalize_col, reference_frame, penalize_mapped))
    }

    
    fn get_average_closest_from_kdtree(&self, kd_tree: &KdTree<Point>, penalize_coor: f32, penalize_col: f32, reference_frame: &mut Vec<Point>, penalize_mapped: f32) -> Point {
        self.get_average_closest(self.get_nearests(kd_tree, 400), penalize_coor, penalize_col, reference_frame, penalize_mapped)
    }
}

impl ply::PropertyAccess for Point {
    fn new() -> Self {
        Point::new_default()
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("x", ply::Property::Float(v)) => self.point_coord.x = v,
            ("y", ply::Property::Float(v)) => self.point_coord.y = v,
            ("z", ply::Property::Float(v)) => self.point_coord.z = v,
            ("red", ply::Property::UChar(v)) => self.point_color.red = v,
            ("green", ply::Property::UChar(v)) => self.point_color.green = v,
            ("blue", ply::Property::UChar(v)) => self.point_color.blue = v,
            (k, _) => panic!("Vertex: Unexpected key/value combination: key: {}", k),
        }
    }
}

impl KdPoint for Point {
    type Scalar = f32;
    type Dim = typenum::U3; // 3 dimensional tree.
    fn at(&self, k: usize) -> f32 { 
        match k {
            0 => self.point_coord.x,
            1 => self.point_coord.y,
            2 => self.point_coord.z,
            _ => panic!("Oh no, don't have {}", k),
        }
    }
}