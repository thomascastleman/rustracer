use crate::scene::Material;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct ComponentIntersection {
    pub t: f32,
    pub normal: glm::Vec4,
    pub uv: (f32, f32),
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

#[derive(Debug)]
pub struct Intersection<'a> {
    pub component_intersection: ComponentIntersection,
    pub material: &'a Material,
}

impl Ord for Intersection<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.component_intersection
            .cmp(&other.component_intersection)
    }
}

impl PartialOrd for Intersection<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Intersection<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.component_intersection == other.component_intersection
    }
}

impl Eq for Intersection<'_> {}
