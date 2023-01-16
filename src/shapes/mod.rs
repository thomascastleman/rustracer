use crate::intersection::{ComponentIntersection, Intersection};
use crate::scene::Material;
pub struct Ray {
    position: glm::Vec4,
    direction: glm::Vec4,
}
impl Ray {
    fn transform(&self, transformation: &glm::Mat4, normalize_direction: bool) -> Ray {
        let position = transformation.mul_v(&self.position);
        let mut direction = transformation.mul_v(&self.direction);

        if normalize_direction {
            direction = glm::normalize(direction);
        }

        Ray {
            position,
            direction,
        }
    }
    pub fn to_object_space(&self, transformation: &glm::Mat4) -> Ray {
        self.transform(transformation, false)
    }
    pub fn to_world_space(&self, transformation: &glm::Mat4) -> Ray {
        self.transform(transformation, true)
    }
    pub fn at(&self, t: f32) -> glm::Vec4 {
        self.position + self.direction * t
    }
}

struct Shape {
    components: Vec<Box<dyn ShapeComponent>>,
    material: Material,
    ctm: glm::Mat4,
}

impl Shape {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let object_ray = ray.to_object_space(&self.ctm);

        let mut intersections: Vec<ComponentIntersection> = Vec::new();
        for component in self.components {
            let object_intersection: Option<ComponentIntersection> =
                component.intersect(&object_ray);

            if let Some(intersection) = object_intersection {
                intersections.push(intersection);
            }
        }

        let closest = intersections.into_iter().min()?;

        Some(Intersection {
            component_intersection: closest,
            material: self.material,
        })
    }
}

trait ShapeComponent {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection>;
}

struct Plane {
    normal: glm::Vec3,
    elevation: f32,
}

impl Plane {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let normal_axis_index = self
            .normal
            .as_array()
            .iter()
            .position(|&axis| axis != 0.0)?;
        let ray_position_on_plane = ray.position.as_array()[normal_axis_index];
        let ray_direction_on_plane = ray.direction.as_array()[normal_axis_index];

        let t = (self.elevation - ray_position_on_plane) / ray_direction_on_plane;

        // TODO: UV mapping has more complexity than this
        let mut uv = ray.at(t).as_array().to_vec();
        uv.remove(normal_axis_index);

        Some(ComponentIntersection {
            t,
            normal: self.normal.extend(0.0),
            uv: (uv[0], uv[1]),
        })
    }
}

struct Square {
    plane: Plane,
}

impl ShapeComponent for Square {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let intersection = self.plane.intersect(ray);

        // Check square constraints

        return Some(ComponentIntersection { t, normal, uv });
    }
}

struct Circle {
    plane: Plane,
}

impl ShapeComponent for Circle {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let (t, normal, uv) = self.plane.intersect(ray);

        // Check circle constraints

        return ComponentIntersection { t, normal, uv };
    }
}

impl<T: QuadraticBody> ShapeComponent for T {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        // calculate the a, b, c
        let quadratic = self.calculate_quadratic(ray);
        let solution = solve_quadratic(quadratic)
            .iter()
            .filter(|t| self.check_constraint(ray.at(t)))
            .min()?;
        Some(ComponentIntersection {
            normal: self.get_normal(ray),
            uv: self.get_uv(),
            t: solution,
        })
    }
}

fn solve_quadratic((a, b, c): (f32, f32, f32)) -> Vec<f32> {
    todo!()
}

trait QuadraticBody {
    fn calculate_quadratic(&self, ray: &Ray) -> (f32, f32, f32);
    fn check_constraint(point: &glm::Vec4) -> bool;
    fn get_normal(point: glm::Vec4) -> glm::Vec4;
    fn get_uv(point: glm::Vec4) -> (f32, f32);
}
