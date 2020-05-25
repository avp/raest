use super::bvh::BVHNode;
use super::rect::*;
use super::*;
use crate::material::Material;
use std::ops::Range;

pub struct Block {
    material: Arc<Material>,
    sides: Arc<BVHNode>,
}

impl Block {
    pub fn new(material: Arc<Material>, p1: Point, p2: Point) -> Arc<Block> {
        let mut sides: Vec<Arc<dyn Hittable>> = vec![];
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

        Arc::new(Block {
            material,
            sides: BVHNode::from_hittables(sides),
        })
    }
}

impl Hittable for Block {
    fn is_light(&self) -> bool {
        match self.material.as_ref() {
            Material::Emission(..) => true,
            _ => false,
        }
    }

    fn bounding_box(&self) -> AABB {
        self.sides.bounding_box()
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        self.sides.hit(ray, range)
    }
}
