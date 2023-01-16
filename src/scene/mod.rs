use crate::shapes::{Primitive, Shape};
use num_traits::identities::One;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

mod parser;

#[derive(Debug)]
enum Light {
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
struct GlobalLightingCoefficients {
    ka: f32,
    kd: f32,
    ks: f32,
}

#[derive(Debug)]
struct Camera {
    position: glm::Vector4<f32>,
    look: glm::Vector4<f32>,
    up: glm::Vector4<f32>,
    height_angle: f32,
}

#[derive(Debug)]
struct Texture {
    filename: PathBuf,
    repeat_u: f32,
    repeat_v: f32,
    blend: f32,
}

#[derive(Debug)]
pub struct Material {
    ambient: glm::Vector4<f32>,
    diffuse: glm::Vector4<f32>,
    specular: glm::Vector4<f32>,
    shininess: f32,
    reflective: glm::Vector4<f32>,
    texture: Option<Texture>,
}

#[derive(Debug)]
enum PrimitiveType {
    Cone,
    Cube,
    Cylinder,
    Sphere,
}

#[derive(Debug)]
pub struct ParsedShape {
    material: Material,
    primitive_type: PrimitiveType,
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

pub struct Scene {
    global_lighting_coefficients: GlobalLightingCoefficients,
    camera: Camera,
    lights: Vec<Light>,
    shapes: Vec<Shape>,
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
            shapes.push(Shape::from_parsed_shape(parsed_shape, primitives));
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

pub struct Primitives {
    cube: Rc<Primitive>,
    sphere: Rc<Primitive>,
    cylinder: Rc<Primitive>,
    cone: Rc<Primitive>,
}

impl Primitives {
    fn new() -> Self {
        todo!()
    }
}
