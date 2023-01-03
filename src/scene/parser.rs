use super::GlobalLightingCoefficients;
use crate::scene::{Camera, Light, Scene};
use anyhow::Result;
use anyhow::{anyhow, bail};
use std::fs::File;
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

    for child in child_elements(&element) {
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
    let mut camera = Camera {
        position: glm::vec4(5.0, 5.0, 5.0, 1.0),
        up: glm::vec4(0.0, 1.0, 0.0, 0.0),
        look: glm::vec4(-1.0, -1.0, -1.0, 0.0),
        height_angle: glm::radians(45.0),
    };

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
            other_name => bail!("Unknown camera tagname: <{}>", other_name),
        }
    }

    if focus_found && look_found {
        bail!("Camera cannot have both focus and look");
    }

    if focus_found {
        camera.look = camera.look - camera.position;
    }

    Ok(camera)
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
                color = Some(
                    parse_vec3(child, ("x", "y", "z"))
                        .or_else(|_| parse_vec3(child, ("r", "g", "b")))?
                        .extend(1.0),
                );
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

    match light_type.as_ref().map(|s| s.as_str()) {
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

impl Scene {
    pub fn parse(filename: &str) -> Result<Self> {
        let root = Element::parse(File::open(filename)?)?;

        if root.name != "scenefile" {
            bail!("Missing <scenefile> tag");
        }

        let mut global_lighting_coefficients = None;
        let mut camera = None;
        let mut lights = Vec::new();

        for child in child_elements(&root) {
            match child.name.as_str() {
                "cameradata" => camera = Some(parse_camera(child)?),
                "lightdata" => lights.push(parse_light(child)?),
                "globaldata" => {
                    global_lighting_coefficients = Some(parse_global_lighting_coefficients(child)?);
                }
                "object" => {}
                other_name => bail!("Unknown tagname <{}>", other_name),
            }
        }

        println!("{:?}", global_lighting_coefficients);
        println!("{:?}", camera);
        println!("{:?}", lights);

        todo!()
    }
}
