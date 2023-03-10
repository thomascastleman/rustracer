use crate::shapes::{
    Axis, Circle, ConeBody, CylinderBody, Plane, Primitive, PrimitiveComponent, Shape, Sphere,
    Square,
};
use num_traits::identities::One;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

mod parser;

#[derive(Debug)]
pub enum Light {
    Point {
        color: glm::Vector4<f32>,
        position: glm::Vector4<f32>,
        attenuation: glm::Vector3<f32>,
    },
    Directional {
        color: glm::Vector4<f32>,
        direction: glm::Vector4<f32>,
        attenuation: glm::Vector3<f32>,
    },
    Spot {
        color: glm::Vector4<f32>,
        position: glm::Vector4<f32>,
        direction: glm::Vector4<f32>,
        attenuation: glm::Vector3<f32>,
        penumbra: f32,
        angle: f32,
    },
}

#[derive(Debug)]
pub struct GlobalLightingCoefficients {
    ka: f32,
    kd: f32,
    ks: f32,
}

#[derive(Debug)]
pub struct Camera {
    position: glm::Vector4<f32>,
    look: glm::Vector4<f32>,
    up: glm::Vector4<f32>,
    pub height_angle: f32,
}

impl Camera {
    fn inverse_view_matrix(&self) -> glm::Mat4 {
        let w = glm::normalize(-self.look).truncate(3);
        let v = glm::normalize(self.up.truncate(3) - (w * glm::dot(self.up.truncate(3), w)));
        let u = glm::cross(v, w);

        let rotation_columns = [
            glm::vec4(u.x, v.x, w.x, 0.0),
            glm::vec4(u.y, v.y, w.y, 0.0),
            glm::vec4(u.z, v.z, w.z, 0.0),
            glm::vec4(0.0, 0.0, 0.0, 1.0),
        ];
        let rotation_matrix = glm::Mat4::from_array(&rotation_columns);

        let rotate_and_translate_matrix =
            glm::ext::translate(rotation_matrix, -self.position.truncate(3));

        rotate_and_translate_matrix
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    filename: PathBuf,
    repeat_u: f32,
    repeat_v: f32,
    blend: f32,
}

#[derive(Debug, Clone)]
pub struct Material {
    ambient: glm::Vector4<f32>,
    diffuse: glm::Vector4<f32>,
    specular: glm::Vector4<f32>,
    shininess: f32,
    reflective: glm::Vector4<f32>,
    texture: Option<Texture>,
}

#[derive(Debug)]
pub enum PrimitiveType {
    Cone,
    Cube,
    Cylinder,
    Sphere,
}

#[derive(Debug)]
pub struct ParsedShape {
    pub material: Material,
    pub primitive_type: PrimitiveType,
}

#[derive(Debug)]
enum Transformation {
    Translate(glm::Vector3<f32>),
    Scale(glm::Vector3<f32>),
    Rotate(glm::Vector3<f32>, f32),
}

impl Transformation {
    fn apply_matrix(&self, ctm: &glm::Mat4) -> glm::Mat4 {
        match self {
            Transformation::Translate(translation) => glm::ext::translate(ctm, *translation),
            Transformation::Rotate(axis, angle) => glm::ext::rotate(ctm, *angle, *axis),
            Transformation::Scale(scale_factors) => glm::ext::scale(ctm, *scale_factors),
        }
    }
}

#[derive(Debug, Default)]
struct Node {
    transformations: Vec<Transformation>,
    shapes: Vec<ParsedShape>,
    children: Vec<Rc<RefCell<Node>>>,
}

#[derive(Debug)]
pub struct TreeScene {
    global_lighting_coefficients: GlobalLightingCoefficients,
    camera: Camera,
    lights: Vec<Light>,
    root_node: Node,
}

#[derive(Debug)]
pub struct Scene {
    pub global_lighting_coefficients: GlobalLightingCoefficients,
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub shapes: Vec<Shape>,
    primitives: Primitives,
}

impl Scene {
    fn traverse_tree_scene<N>(
        node: N,
        primitives: &Primitives,
        shapes: &mut Vec<Shape>,
        mut ctm: glm::Mat4,
    ) where
        N: std::ops::Deref<Target = Node>,
    {
        for transformation in &node.transformations {
            ctm = transformation.apply_matrix(&ctm);
        }

        for parsed_shape in &node.shapes {
            shapes.push(Shape::from_parsed_shape(parsed_shape, primitives, ctm));
        }

        for child in &node.children {
            Scene::traverse_tree_scene(child.borrow(), primitives, shapes, ctm);
        }
    }
}

impl From<TreeScene> for Scene {
    fn from(tree_scene: TreeScene) -> Self {
        let primitives = Primitives::new();

        // Traverse the scene's node tree and construct shapes from it, using
        // the transformations at each node to add CTMs to the shapes.
        let mut shapes = Vec::new();
        Scene::traverse_tree_scene(
            &tree_scene.root_node,
            &primitives,
            &mut shapes,
            glm::Mat4::one(),
        );

        Scene {
            global_lighting_coefficients: tree_scene.global_lighting_coefficients,
            camera: tree_scene.camera,
            lights: tree_scene.lights,
            shapes,
            primitives,
        }
    }
}

#[derive(Debug)]
pub struct Primitives {
    pub cube: Rc<Primitive>,
    pub sphere: Rc<Primitive>,
    pub cylinder: Rc<Primitive>,
    pub cone: Rc<Primitive>,
}

impl Primitives {
    fn new() -> Self {
        let mut cube_components: Vec<Box<dyn PrimitiveComponent>> = Vec::new();
        for &normal_axis in Axis::iterator() {
            for elevation in [-0.5, 0.5] {
                cube_components.push(Box::new(Square {
                    plane: Plane {
                        normal_axis,
                        elevation,
                    },
                }))
            }
        }

        Self {
            cube: Rc::new(Primitive {
                components: cube_components,
            }),
            sphere: Rc::new(Primitive {
                components: vec![Box::new(Sphere {})],
            }),
            cylinder: Rc::new(Primitive {
                components: vec![
                    Box::new(CylinderBody {}),
                    Box::new(Circle {
                        plane: Plane {
                            normal_axis: Axis::Y,
                            elevation: -0.5,
                        },
                    }),
                ],
            }),
            cone: Rc::new(Primitive {
                components: vec![
                    Box::new(ConeBody {}),
                    Box::new(Circle {
                        plane: Plane {
                            normal_axis: Axis::Y,
                            elevation: -0.5,
                        },
                    }),
                ],
            }),
        }
    }
}
