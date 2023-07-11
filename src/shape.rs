use crate::intersection::Intersection;
use crate::primitive::Primitive;
use crate::raytracer::Ray;
use crate::scene::{Material, ParsedShape, PrimitiveType, Primitives};
use std::rc::Rc;

/// A Shape represents a particular instance of a Primitive, which has been
/// transformed and has a material (which affects lighting).
#[derive(Debug)]
pub struct Shape {
    /// Reference to the primitive shape that this is an instance of.
    primitive: Rc<Primitive>,
    /// Material of this particular shape.
    pub material: Material,
    /// The cumulative transformation matrix for this shape.
    ctm: glm::Mat4,
}

impl Shape {
    pub fn from_parsed_shape(
        parsed_shape: &ParsedShape,
        primitives: &Primitives,
        ctm: glm::Mat4,
    ) -> Self {
        let primitive = Rc::clone(match parsed_shape.primitive_type {
            PrimitiveType::Cone => &primitives.cone,
            PrimitiveType::Cube => &primitives.cube,
            PrimitiveType::Sphere => &primitives.sphere,
            PrimitiveType::Cylinder => &primitives.cylinder,
        });

        // TODO: Instead of cloning the material here, we could have it be multiply-owned (Rc)
        Self {
            primitive,
            material: parsed_shape.material.clone(),
            ctm,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let inverse_ctm = glm::inverse(&self.ctm);
        let object_space_ray = ray.to_object_space(&inverse_ctm);

        let mut component_intersection = self.primitive.intersect(&object_space_ray)?;

        let four_ctm_vec3s = self.ctm.as_array().map(|v| v.truncate(3));
        let three_ctm_vec3s = [four_ctm_vec3s[0], four_ctm_vec3s[1], four_ctm_vec3s[2]];
        let ctm_mat3 = glm::Mat3::from_array(&three_ctm_vec3s);
        let ctm_mat3_transpose = glm::transpose(ctm_mat3);
        let normal_transform = glm::inverse(&ctm_mat3_transpose);
        let world_normal =
            glm::normalize(normal_transform * component_intersection.normal.truncate(3))
                .extend(0.0);

        component_intersection.normal = world_normal;

        Some(Intersection {
            component_intersection,
            material: &self.material,
        })
    }
}
