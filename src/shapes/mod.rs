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
        for component in &self.components {
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
            .position(|&axis| axis != 0.0)
            .unwrap();

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
        todo!()
    }
}

struct Circle {
    plane: Plane,
}

impl ShapeComponent for Circle {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let intersection = self.plane.intersect(ray);

        // Check circle constraints
        todo!()
    }
}

impl<T: QuadraticBody> ShapeComponent for T {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let (a, b, c) = self.calculate_quadratic_coefficients(ray);

        let solution = solve_quadratic(a, b, c)
            .into_iter()
            .filter(|&t| self.check_constraint(&ray.at(t)))
            .reduce(f32::min)?;

        let intersection_point = ray.at(solution);

        Some(ComponentIntersection {
            normal: self.normal_at_intersection(&intersection_point),
            uv: self.uv_at_intersection(&intersection_point),
            t: solution,
        })
    }
}

/// Finds all real solutions to a quadratic equation defined by coefficients a, b, and c.
fn solve_quadratic(a: f32, b: f32, c: f32) -> Vec<f32> {
    let mut solutions = Vec::new();
    let discriminant = b.powi(2) - (4.0 * a * c);

    if discriminant >= 0.0 {
        let root = discriminant.sqrt();
        let double_a = 2.0 * a;
        let t1 = (-b + root) / double_a;
        let t2 = (-b - root) / double_a;

        solutions.push(t1);

        // If the discriminant is 0, then t1 = t2 (multiple root), so no need to include it twice
        if discriminant != 0.0 {
            solutions.push(t2);
        }
    }

    return solutions;
}

/// Trait that unifies all shape components whose intersections are computed using a
/// quadratic function. This includes the cone body, cylinder body, and entire sphere.
trait QuadraticBody {
    /// Uses the given ray's position/direction to calculate a quadratic equation whose
    /// solutions represent intersections with the shape component.
    fn calculate_quadratic_coefficients(&self, ray: &Ray) -> (f32, f32, f32);

    /// Determines whether or not a given point of intersection actually lies
    /// within the bounds of the shape component.
    fn check_constraint(&self, point: &glm::Vec4) -> bool;

    /// Finds the normal vector to the shape component at a given point on the shape component.
    fn normal_at_intersection(&self, point: &glm::Vec4) -> glm::Vec4;

    /// Finds the UV coordinate at a given point on the shape component.
    fn uv_at_intersection(&self, point: &glm::Vec4) -> (f32, f32);
}
