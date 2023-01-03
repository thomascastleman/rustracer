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

struct Texture {
    filename: String,
    repeat_u: f32,
    repeat_v: f32,
    blend: f32,
}

struct Material {
    ambient: glm::Vector4<f32>,
    diffuse: glm::Vector4<f32>,
    specular: glm::Vector4<f32>,
    shininess: f32,
    reflective: glm::Vector4<f32>,
    texture: Option<Texture>,
}

enum PrimitiveType {
    Cone,
    Cube,
    Cylinder,
    Sphere,
}

struct Primitive {
    material: Material,
    primitive_type: PrimitiveType,
}

enum Transformation {
    Translate(glm::Vector3<f32>),
    Scale(glm::Vector3<f32>),
    Rotate(glm::Vector3<f32>, f32),
}

struct Node {
    transformations: Vec<Transformation>,
    primitives: Vec<Primitive>,
    children: Vec<Node>,
}

pub struct Scene {
    global_lighting_coefficients: GlobalLightingCoefficients,
    camera: Camera,
    lights: Vec<Light>,
    root_node: Node,
}
