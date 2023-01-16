use crate::scene::Material;
use std::cmp::Ordering;

pub struct ComponentIntersection {
    t: f32,
    normal: glm::Vec4,
    uv: (f32, f32),
}

impl Ord for ComponentIntersection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.t.partial_cmp(&other.t).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for ComponentIntersection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ComponentIntersection {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

impl Eq for ComponentIntersection {}

pub struct Intersection {
    component_intersection: ComponentIntersection,
    material: Material,
}

impl Ord for Intersection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.component_intersection
            .cmp(&other.component_intersection)
    }
}

impl PartialOrd for Intersection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Intersection {
    fn eq(&self, other: &Self) -> bool {
        self.component_intersection == other.component_intersection
    }
}

impl Eq for Intersection {}
