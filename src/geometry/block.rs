use super::bvh::BVHNode;
use super::rect::*;
use super::*;
use crate::material::Material;
use std::ops::Range;

pub struct Block {
    sides: Arc<BVHNode>,
}

impl Block {
    pub fn new(material: Material, p1: Point, p2: Point) -> Box<Block> {
        let mut sides: Vec<Box<dyn Hittable>> = vec![];
        sides.push(Rect::new(
            material.clone(),
            RectAxis::XY,
            (p1.x, p1.y),
            (p2.x, p2.y),
            p1.z,
        ));
        sides.push(Rect::new(
            material.clone(),
            RectAxis::XY,
            (p1.x, p1.y),
            (p2.x, p2.y),
            p2.z,
        ));

        sides.push(Rect::new(
            material.clone(),
            RectAxis::XZ,
            (p1.x, p1.z),
            (p2.x, p2.z),
            p1.y,
        ));
        sides.push(Rect::new(
            material.clone(),
            RectAxis::XZ,
            (p1.x, p1.z),
            (p2.x, p2.z),
            p2.y,
        ));

        sides.push(Rect::new(
            material.clone(),
            RectAxis::YZ,
            (p1.y, p1.z),
            (p2.y, p2.z),
            p1.x,
        ));
        sides.push(Rect::new(
            material.clone(),
            RectAxis::YZ,
            (p1.y, p1.z),
            (p2.y, p2.z),
            p2.x,
        ));

        Box::new(Block {
            sides: BVHNode::from_hittables(sides),
        })
    }
}

impl Hittable for Block {
    fn bounding_box(&self) -> AABB {
        self.sides.bounding_box()
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        self.sides.hit(ray, range)
    }
}
