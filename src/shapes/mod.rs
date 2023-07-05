use crate::intersection::{ComponentIntersection, Intersection};
use crate::scene::{Material, ParsedShape, PrimitiveType, Primitives};
use std::f32::consts::PI;
use std::rc::Rc;
use std::slice::Iter;

#[derive(Debug)]
pub struct Ray {
    pub position: glm::Vec4,
    pub direction: glm::Vec4,
}

impl Ray {
    /// Constructs a new Ray from the given components.
    pub fn new(position: glm::Vec4, direction: glm::Vec4) -> Self {
        Self {
            position,
            direction,
        }
    }

    pub fn transform(&self, transformation: &glm::Mat4, normalize_direction: bool) -> Ray {
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

/// A Primitive is a object-space version of a Shape, which represents the
/// geometry of that shape. Primitives are composed of components (for instance
/// a cube is composed of 6 plane components). All shape instances of the same
/// kind of shape share a Primitive.
#[derive(Debug)]
pub struct Primitive {
    pub components: Vec<Box<dyn PrimitiveComponent>>,
}

impl Primitive {
    fn intersect(&self, object_space_ray: &Ray) -> Option<ComponentIntersection> {
        let mut intersections: Vec<ComponentIntersection> = Vec::new();

        for component in &self.components {
            let object_intersection: Option<ComponentIntersection> =
                component.intersect(object_space_ray);

            if let Some(intersection) = object_intersection {
                intersections.push(intersection);
            }
        }

        intersections.into_iter().min()
    }
}

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

pub trait PrimitiveComponent: std::fmt::Debug {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection>;
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub fn iterator() -> Iter<'static, Axis> {
        static AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
        AXES.iter()
    }
}

#[derive(Debug)]
pub struct Plane {
    pub normal_axis: Axis,
    pub elevation: f32,
}

impl Plane {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let ray_position_on_plane = ray.position.as_array()[self.normal_axis as usize];
        let ray_direction_on_plane = ray.direction.as_array()[self.normal_axis as usize];

        if ray_direction_on_plane == 0.0 {
            return None;
        }

        let t = (self.elevation - ray_position_on_plane) / ray_direction_on_plane;

        // Reject negative t-values which represent aiming in the opposite direction of the ray
        if t < 0.0 {
            return None;
        }

        let uv = self.uv_map(&ray.at(t));

        Some(ComponentIntersection {
            t,
            normal: self.normal(),
            uv,
        })
    }

    fn uv_map(&self, point: &glm::Vec4) -> (f32, f32) {
        let prescaled = match self.normal_axis {
            Axis::X => {
                if self.elevation > 0.0 {
                    (-point.z, point.y)
                } else {
                    (point.z, point.y)
                }
            }
            Axis::Y => {
                if self.elevation > 0.0 {
                    (point.x, -point.z)
                } else {
                    (point.x, point.z)
                }
            }
            Axis::Z => {
                if self.elevation > 0.0 {
                    (point.x, point.y)
                } else {
                    (-point.x, point.y)
                }
            }
        };

        (prescaled.0 + 0.5, prescaled.1 + 0.5)
    }

    fn normal(&self) -> glm::Vec4 {
        let mut normal = glm::vec4(0.0, 0.0, 0.0, 0.0);
        normal[self.normal_axis as usize] = 1.0;
        normal
    }

    /// Flattens a point in 3D space onto this plane, returning a 2D point.
    fn flatten_onto(&self, point: &glm::Vec4) -> [f32; 2] {
        match self.normal_axis {
            Axis::X => [point.y, point.z],
            Axis::Y => [point.x, point.z],
            Axis::Z => [point.x, point.y],
        }
    }
}

#[derive(Debug)]
pub struct Square {
    pub plane: Plane,
}

impl PrimitiveComponent for Square {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let intersection = self.plane.intersect(ray)?;
        let intersection_point = ray.at(intersection.t);
        let flattened_intersection_point = self.plane.flatten_onto(&intersection_point);

        fn within_square(v: f32) -> bool {
            (-0.5..=0.5).contains(&v)
        }

        if flattened_intersection_point.into_iter().all(within_square) {
            Some(intersection)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Circle {
    pub plane: Plane,
}

impl PrimitiveComponent for Circle {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let intersection = self.plane.intersect(ray)?;
        let intersection_point = ray.at(intersection.t);
        let [horizontal, vertical] = self.plane.flatten_onto(&intersection_point);

        if horizontal.powi(2) + vertical.powi(2) <= 0.5f32.powi(2) {
            Some(intersection)
        } else {
            None
        }
    }
}

impl<T: QuadraticBody + std::fmt::Debug> PrimitiveComponent for T {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let (a, b, c) = self.calculate_quadratic_coefficients(ray);

        let solution = solve_quadratic(a, b, c)
            .into_iter()
            .filter(|&t| t >= 0.0 && self.check_constraint(&ray.at(t)))
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

    solutions
}

/// Trait that unifies all shape components whose intersections are computed using a
/// quadratic function. This includes the cone body, cylinder body, and entire sphere.
trait QuadraticBody {
    /// Uses the given ray's position/direction to calculate a quadratic equation whose
    /// solutions represent intersections with the shape component.
    fn calculate_quadratic_coefficients(&self, ray: &Ray) -> (f32, f32, f32);

    /// Determines whether or not a given point of intersection actually lies
    /// within the bounds of the shape component.
    fn check_constraint(&self, point: &glm::Vec4) -> bool {
        -0.5 <= point.y && point.y <= 0.5
    }

    /// Finds the normal vector to the shape component at a given point on the shape component.
    fn normal_at_intersection(&self, point: &glm::Vec4) -> glm::Vec4;

    /// Finds the UV coordinate at a given point on the shape component.
    fn uv_at_intersection(&self, point: &glm::Vec4) -> (f32, f32);
}

#[derive(Debug)]
pub struct ConeBody;

impl QuadraticBody for ConeBody {
    fn calculate_quadratic_coefficients(&self, ray: &Ray) -> (f32, f32, f32) {
        let a = ray.direction.x.powi(2) + ray.direction.z.powi(2)
            - (1.0 / 4.0) * ray.direction.y.powi(2);
        let b = (2.0 * ray.position.x * ray.direction.x)
            + (2.0 * ray.position.z * ray.direction.z)
            + ((1.0 / 4.0) * ray.direction.y)
            - ((1.0 / 2.0) * ray.position.y * ray.direction.y);
        let c = ray.position.x.powi(2) + ray.position.z.powi(2) + ((1.0 / 4.0) * ray.position.y)
            - (1.0 / 4.0) * ray.position.y.powi(2)
            - (1.0 / 16.0);

        (a, b, c)
    }

    fn normal_at_intersection(&self, point: &glm::Vec4) -> glm::Vec4 {
        let x_norm = 2.0 * point.x;
        let y_norm = -(1.0 / 4.0) * (2.0 * point.y - 1.0);
        let z_norm = 2.0 * point.z;

        glm::vec4(x_norm, y_norm, z_norm, 0.0)
    }

    fn uv_at_intersection(&self, point: &glm::Vec4) -> (f32, f32) {
        let theta = point.z.atan2(point.x);
        let u = if theta < 0.0 {
            -theta / (2.0 * PI)
        } else {
            1.0 - (theta / (2.0 * PI))
        };

        (u, point.y + 0.5)
    }
}

#[derive(Debug)]
pub struct CylinderBody;

impl QuadraticBody for CylinderBody {
    fn calculate_quadratic_coefficients(&self, ray: &Ray) -> (f32, f32, f32) {
        let a = ray.direction.x.powi(2) + ray.direction.z.powi(2);
        let b = 2.0 * (ray.position.x * ray.direction.x + ray.position.z * ray.direction.z);
        let c = ray.position.x.powi(2) + ray.position.z.powi(2) - 0.5f32.powi(2);

        (a, b, c)
    }

    fn normal_at_intersection(&self, point: &glm::Vec4) -> glm::Vec4 {
        glm::vec4(2.0 * point.x, 0.0, 2.0 * point.z, 0.0)
    }

    fn uv_at_intersection(&self, point: &glm::Vec4) -> (f32, f32) {
        let theta = point.z.atan2(point.x);
        let u = if theta < 0.0 {
            -theta / (2.0 * PI)
        } else {
            1.0 - (theta / (2.0 * PI))
        };

        (u, point.y + 0.5)
    }
}

#[derive(Debug)]
pub struct Sphere;

impl QuadraticBody for Sphere {
    fn calculate_quadratic_coefficients(&self, ray: &Ray) -> (f32, f32, f32) {
        let a = ray.direction.x.powi(2) + ray.direction.y.powi(2) + ray.direction.z.powi(2);
        let b = 2.0
            * (ray.position.x * ray.direction.x
                + ray.position.y * ray.direction.y
                + ray.position.z * ray.direction.z);
        let c = ray.position.x.powi(2) + ray.position.y.powi(2) + ray.position.z.powi(2)
            - 0.5f32.powi(2);

        (a, b, c)
    }

    fn normal_at_intersection(&self, point: &glm::Vec4) -> glm::Vec4 {
        glm::vec4(2.0 * point.x, 2.0 * point.y, 2.0 * point.z, 0.0)
    }

    fn uv_at_intersection(&self, point: &glm::Vec4) -> (f32, f32) {
        let v = (point.y / 0.5).asin() / PI + 0.5;

        let u = if v == 0.0 || v == 1.0 {
            0.5
        } else {
            let theta = point.z.atan2(point.x);
            if theta < 0.0 {
                -theta / (2.0 * PI)
            } else {
                1.0 - (theta / (2.0 * PI))
            }
        };

        (u, v)
    }
}
