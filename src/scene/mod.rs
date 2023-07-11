use crate::lights::Light;
use crate::primitive::{
    Axis, Circle, ConeBody, CylinderBody, Plane, Primitive, PrimitiveComponent, Sphere, Square,
};
use crate::shape::Shape;
use image::RgbImage;
use num_traits::identities::One;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

mod parser;

#[derive(Debug)]
pub struct GlobalLightingCoefficients {
    pub ka: f32,
    pub kd: f32,
    pub ks: f32,
}

#[derive(Debug)]
pub struct Camera {
    position: glm::Vector4<f32>,
    look: glm::Vector4<f32>,
    up: glm::Vector4<f32>,
    pub height_angle: f32,
    pub inverse_view_matrix: glm::Mat4,
}

impl Camera {
    pub fn new(position: glm::Vec4, look: glm::Vec4, up: glm::Vec4, height_angle: f32) -> Self {
        Self {
            position,
            look,
            up,
            height_angle,
            inverse_view_matrix: Camera::calculate_inverse_view_matrix(position, look, up),
        }
    }

    fn calculate_inverse_view_matrix(
        position: glm::Vec4,
        look: glm::Vec4,
        up: glm::Vec4,
    ) -> glm::Mat4 {
        let w = glm::normalize(-look).truncate(3);
        let v = glm::normalize(up.truncate(3) - (w * glm::dot(up.truncate(3), w)));
        let u = glm::cross(v, w);

        let rotation_matrix = glm::Mat4::new(
            glm::vec4(u.x, v.x, w.x, 0.0),
            glm::vec4(u.y, v.y, w.y, 0.0),
            glm::vec4(u.z, v.z, w.z, 0.0),
            glm::vec4(0.0, 0.0, 0.0, 1.0),
        );

        let translation_matrix = glm::Mat4::new(
            glm::vec4(1.0, 0.0, 0.0, 0.0),
            glm::vec4(0.0, 1.0, 0.0, 0.0),
            glm::vec4(0.0, 0.0, 1.0, 0.0),
            glm::vec4(-position.x, -position.y, -position.z, 1.0),
        );

        let rotate_and_translate_matrix = rotation_matrix * translation_matrix;

        glm::inverse(&rotate_and_translate_matrix)
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub filename: PathBuf,
    pub repeat_u: f32,
    pub repeat_v: f32,
    pub blend: f32,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub ambient: glm::Vector4<f32>,
    pub diffuse: glm::Vector4<f32>,
    pub specular: glm::Vector4<f32>,
    pub shininess: f32,
    pub reflective: glm::Vector4<f32>,
    pub texture: Option<Texture>,
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
    pub textures: HashMap<PathBuf, RgbImage>,
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

impl TryFrom<TreeScene> for Scene {
    type Error = anyhow::Error;

    fn try_from(tree_scene: TreeScene) -> std::result::Result<Self, Self::Error> {
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

        let mut textures = HashMap::new();
        for shape in &shapes {
            if let Some(ref texture) = shape.material.texture {
                if !textures.contains_key(&texture.filename) {
                    let texture_image = image::open(&texture.filename)?.to_rgb8();
                    textures.insert(texture.filename.clone(), texture_image);
                }
            }
        }

        Ok(Scene {
            global_lighting_coefficients: tree_scene.global_lighting_coefficients,
            camera: tree_scene.camera,
            lights: tree_scene.lights,
            shapes,
            textures,
        })
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
                            elevation: 0.5,
                        },
                    }),
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
