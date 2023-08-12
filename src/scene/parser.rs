//! Parser for XML scenefiles.

use super::{GlobalLightingCoefficients, Material, Node, ParsedShape, PrimitiveType, Texture};
use crate::lights::Light;
use crate::scene::{Camera, Transformation, TreeScene};
use anyhow::Result;
use anyhow::{anyhow, bail};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use xmltree::Element;

fn parse_attribute<T: FromStr>(element: &Element, attribute_name: &str) -> Result<T> {
    element
        .attributes
        .get(attribute_name)
        .ok_or_else(|| {
            anyhow!(
                "<{}> tag must have \"{}\" attribute",
                element.name,
                attribute_name
            )
        })?
        .parse()
        .map_err(|_| {
            anyhow!(
                "Invalid attribute value for tag <{}> and attribute \"{}\"",
                element.name,
                attribute_name,
            )
        })
}

fn parse_vec3(element: &Element, (x, y, z): (&str, &str, &str)) -> Result<glm::Vector3<f32>> {
    Ok(glm::vec3(
        parse_attribute(element, x)?,
        parse_attribute(element, y)?,
        parse_attribute(element, z)?,
    ))
}

fn parse_global_lighting_coefficients(element: &Element) -> Result<GlobalLightingCoefficients> {
    let mut global_lighting_coefficients = GlobalLightingCoefficients {
        ka: 0.5,
        kd: 0.5,
        ks: 0.5,
    };

    for child in child_elements(element) {
        match child.name.as_str() {
            "ambientcoeff" => {
                global_lighting_coefficients.ka = parse_attribute(child, "v")?;
            }
            "diffusecoeff" => {
                global_lighting_coefficients.kd = parse_attribute(child, "v")?;
            }
            "specularcoeff" => {
                global_lighting_coefficients.ks = parse_attribute(child, "v")?;
            }
            other_name => bail!(
                "Unknown global lighting coefficient tagname: <{}>",
                other_name
            ),
        }
    }

    Ok(global_lighting_coefficients)
}

fn child_elements(element: &Element) -> impl Iterator<Item = &Element> {
    element
        .children
        .iter()
        .filter_map(|child| child.as_element())
}

fn parse_camera(element: &Element) -> Result<Camera> {
    let mut camera = Camera::new(
        glm::vec4(5.0, 5.0, 5.0, 1.0),
        glm::vec4(0.0, 1.0, 0.0, 0.0),
        glm::vec4(-1.0, -1.0, -1.0, 0.0),
        glm::radians(45.0),
    );

    let mut look_found = false;
    let mut focus_found = false;

    for child in child_elements(element) {
        match child.name.as_str() {
            "pos" => {
                camera.position = parse_vec3(child, ("x", "y", "z"))?.extend(1.0);
            }
            "up" => {
                camera.up = parse_vec3(child, ("x", "y", "z"))?.extend(0.0);
            }
            "heightangle" => {
                camera.height_angle = glm::radians(parse_attribute(child, "v")?);
            }
            "look" => {
                camera.look = parse_vec3(child, ("x", "y", "z"))?.extend(0.0);
                look_found = true;
            }
            "focus" => {
                camera.look = parse_vec3(child, ("x", "y", "z"))?.extend(1.0);
                focus_found = true;
            }
            unsupported_tagname @ ("aperture" | "focallength") => {
                eprintln!(
                    "Ignoring unsupported camera tagname: <{}>",
                    unsupported_tagname
                );
            }
            other_name => bail!("Unknown camera tagname: <{}>", other_name),
        }
    }

    if focus_found && look_found {
        bail!("Camera cannot have both focus and look");
    }

    if focus_found {
        camera.look = camera.look - camera.position;
    }

    // Ensure that the inverse view matrix has been calculated using the most up-to-date position/look/up
    camera.inverse_view_matrix =
        Camera::calculate_inverse_view_matrix(camera.position, camera.look, camera.up);

    Ok(camera)
}

fn parse_color(element: &Element) -> Result<glm::Vec4> {
    Ok(parse_vec3(element, ("x", "y", "z"))
        .or_else(|_| parse_vec3(element, ("r", "g", "b")))?
        .extend(1.0))
}

fn parse_light(element: &Element) -> Result<Light> {
    let mut color = None;
    let mut direction = None;
    let mut position = None;
    let mut attenuation = None;
    let mut penumbra = None;
    let mut angle = None;
    let mut light_type = None;

    for child in child_elements(element) {
        match child.name.as_str() {
            "id" => {}
            "type" => {
                light_type = Some(parse_attribute::<String>(child, "v")?);
            }
            "color" => {
                color = Some(parse_color(child)?);
            }
            "function" => {
                attenuation = Some(
                    parse_vec3(child, ("a", "b", "c"))
                        .or_else(|_| parse_vec3(child, ("x", "y", "z")))
                        .or_else(|_| parse_vec3(child, ("v1", "v2", "v3")))?,
                );
            }
            "position" => {
                position = Some(parse_vec3(child, ("x", "y", "z"))?.extend(1.0));
            }
            "direction" => {
                direction = Some(parse_vec3(child, ("x", "y", "z"))?.extend(0.0));
            }
            "angle" => {
                angle = Some(glm::radians(parse_attribute::<f32>(child, "v")?));
            }
            "penumbra" => {
                penumbra = Some(glm::radians(parse_attribute::<f32>(child, "v")?));
            }
            other_name => {
                bail!("Unknown light tagname: <{}>", other_name)
            }
        }
    }

    let default_color = glm::vec4(1.0, 1.0, 1.0, 1.0);
    let default_position = glm::vec4(3.0, 3.0, 3.0, 1.0);
    let default_attenuation = glm::vec3(1.0, 0.0, 0.0);
    let default_direction = glm::vec4(0.0, 0.0, 0.0, 0.0);

    match light_type.as_deref() {
        Some("directional") => {
            if position.is_some() {
                bail!("Directional light cannot have position");
            }
            if penumbra.is_some() {
                bail!("Directional light cannot have penumbra");
            }
            if angle.is_some() {
                bail!("Directional light cannot have angle");
            }

            Ok(Light::Directional {
                color: color.unwrap_or(default_color),
                direction: direction.unwrap_or(default_direction),
                attenuation: attenuation.unwrap_or(default_attenuation),
            })
        }
        Some("point") | None => {
            if direction.is_some() {
                bail!("Point light cannot have direction");
            }
            if penumbra.is_some() {
                bail!("Point light cannot have penumbra");
            }
            if angle.is_some() {
                bail!("Point light cannot have angle");
            }

            Ok(Light::Point {
                color: color.unwrap_or(default_color),
                position: position.unwrap_or(default_position),
                attenuation: attenuation.unwrap_or(default_attenuation),
            })
        }
        Some("spot") => Ok(Light::Spot {
            color: color.unwrap_or(default_color),
            position: position.unwrap_or(default_position),
            direction: direction.unwrap_or(default_direction),
            attenuation: attenuation.unwrap_or(default_attenuation),
            penumbra: penumbra.unwrap_or(0.0),
            angle: angle.unwrap_or(0.0),
        }),
        Some(t) => bail!("Unknown light type: \"{}\"", t),
    }
}

/// Map from object names to the node for that object
type ObjectMap = HashMap<String, Rc<RefCell<Node>>>;

fn parse_object_body(
    element: &Element,
    parent_node: &Rc<RefCell<Node>>,
    objects: &ObjectMap,
    textures: &Path,
) -> Result<()> {
    for child in child_elements(element) {
        match child.name.as_str() {
            "transblock" => {
                let child_node: Rc<RefCell<Node>> = Default::default();

                // Add child to parent's children list
                parent_node
                    .borrow_mut()
                    .children
                    .push(Rc::clone(&child_node));

                parse_transblock(child, child_node, objects, textures)?;
            }
            other_name => bail!("Cannot have tag <{}> in <object>", other_name),
        }
    }

    Ok(())
}

fn parse_object(element: &Element, objects: &mut ObjectMap, textures: &Path) -> Result<()> {
    let object_name = parse_attribute::<String>(element, "name")?;
    let object_type = parse_attribute::<String>(element, "type")?;

    if object_type.as_str() != "tree" {
        bail!(
            "Top-level <object> elements must be of type \"tree\", not \"{}\"",
            object_type
        )
    }

    let current_node = Default::default();

    if objects
        .insert(object_name.clone(), Rc::clone(&current_node))
        .is_some()
    {
        bail!(
            "Cannot have two objects with the same name: {}",
            object_name
        );
    }

    parse_object_body(element, &current_node, objects, textures)?;

    Ok(())
}

fn parse_transblock(
    element: &Element,
    node: Rc<RefCell<Node>>,
    objects: &ObjectMap,
    textures: &Path,
) -> Result<()> {
    for child in child_elements(element) {
        match child.name.as_str() {
            "translate" => {
                node.borrow_mut()
                    .transformations
                    .push(Transformation::Translate(parse_vec3(
                        child,
                        ("x", "y", "z"),
                    )?));
            }
            "rotate" => {
                node.borrow_mut()
                    .transformations
                    .push(Transformation::Rotate(
                        parse_vec3(child, ("x", "y", "z"))?,
                        glm::radians(parse_attribute(child, "angle")?),
                    ));
            }
            "scale" => {
                node.borrow_mut()
                    .transformations
                    .push(Transformation::Scale(parse_vec3(child, ("x", "y", "z"))?));
            }
            "object" => match parse_attribute::<String>(child, "type")?.as_str() {
                "master" => {
                    let master_object = objects
                        .get(&parse_attribute::<String>(child, "name")?)
                        .ok_or_else(|| anyhow!("Master object must have name"))?;

                    node.borrow_mut().children.push(Rc::clone(master_object));
                }
                "tree" => parse_object_body(child, &node, objects, textures)?,
                "primitive" => parse_primitive(child, &node, textures)?,
                other_name => bail!("Cannot have tag<{}> in <object>", other_name),
            },
            other_name => bail!("Cannot have tag <{}> in <transblock>", other_name),
        }
    }

    Ok(())
}

fn parse_primitive(element: &Element, node: &Rc<RefCell<Node>>, textures: &Path) -> Result<()> {
    let primitive_type = match parse_attribute::<String>(element, "name")?.as_str() {
        "sphere" => PrimitiveType::Sphere,
        "cube" => PrimitiveType::Cube,
        "cylinder" => PrimitiveType::Cylinder,
        "cone" => PrimitiveType::Cone,
        other_name => bail!("Unsupported primitive type {}", other_name),
    };

    let mut diffuse = None;
    let mut ambient = None;
    let mut specular = None;
    let mut reflective = None;
    let mut shininess = None;
    let mut texture = None;
    let mut blend = None;

    for child in child_elements(element) {
        match child.name.as_str() {
            "diffuse" => diffuse = Some(parse_color(child)?),
            "ambient" => ambient = Some(parse_color(child)?),
            "specular" => specular = Some(parse_color(child)?),
            "reflective" => reflective = Some(parse_color(child)?),
            "shininess" => shininess = Some(parse_attribute::<f32>(child, "v")?),
            "texture" => texture = Some(parse_texture_map(child, textures)?),
            "blend" => blend = Some(parse_attribute::<f32>(child, "v")?),
            other_name => bail!("Cannot have <{}> tag in primitive object", other_name),
        }
    }

    // Add the blend to the texture
    if let Some(ref mut texture) = texture {
        texture.blend = blend.unwrap_or(0.0);
    }

    let zero = glm::vec4(0.0, 0.0, 0.0, 0.0);

    let material = Material {
        ambient: ambient.unwrap_or(zero),
        diffuse: diffuse.unwrap_or(glm::vec4(1.0, 1.0, 1.0, 0.0)),
        specular: specular.unwrap_or(zero),
        shininess: shininess.unwrap_or(0.0),
        reflective: reflective.unwrap_or(zero),
        texture,
    };

    let shape = ParsedShape {
        primitive_type,
        material,
    };

    // Add shape to node's list of shapes
    node.borrow_mut().shapes.push(shape);

    Ok(())
}

fn parse_texture_map(element: &Element, textures: &Path) -> Result<Texture> {
    let filename = Path::join(
        textures,
        Path::new(&parse_attribute::<String>(element, "file")?),
    );

    let repeat_u = parse_attribute(element, "u").unwrap_or(1.0);
    let repeat_v = parse_attribute(element, "v").unwrap_or(1.0);

    Ok(Texture {
        filename,
        repeat_u,
        repeat_v,
        blend: 0.0,
    })
}

impl TreeScene {
    /// Parses a `Scene` from the given scenefile path and a path that all
    /// texture images are relative to.
    pub fn parse(scenefile: &Path, textures: &Path) -> Result<Self> {
        let root = Element::parse(File::open(scenefile)?)?;

        if root.name != "scenefile" {
            bail!("Missing <scenefile> tag");
        }

        let mut global_lighting_coefficients = None;
        let mut camera = None;
        let mut lights = Vec::new();

        let mut objects = HashMap::new();

        for child in child_elements(&root) {
            match child.name.as_str() {
                "cameradata" => camera = Some(parse_camera(child)?),
                "lightdata" => lights.push(parse_light(child)?),
                "globaldata" => {
                    global_lighting_coefficients = Some(parse_global_lighting_coefficients(child)?);
                }
                "object" => parse_object(child, &mut objects, textures)?,
                other_name => bail!("Unknown tagname <{}>", other_name),
            }
        }

        let root_node = objects
            .remove("root")
            .ok_or_else(|| anyhow!("Scene must have a root object"))?;

        // Extract ownership of the root node by unwrapping its RefCell and Rc containers.
        // This works because we have a single Rc<RefCell<Node>> to the root node (it
        // has no parents), which we've just moved out of the objects map.
        let root_node = Rc::try_unwrap(root_node).unwrap().into_inner();

        Ok(TreeScene {
            global_lighting_coefficients: global_lighting_coefficients
                .ok_or_else(|| anyhow!("Must have <globaldata> tag"))?,
            camera: camera.ok_or_else(|| anyhow!("Must have <cameradata> tag"))?,
            lights,
            root_node,
        })
    }
}
