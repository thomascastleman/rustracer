use std::path::Path;

mod scene;

fn main() {
    let scene = scene::Scene::parse(
        Path::new(
            "/Users/smorris/Desktop/Sophomore Year/CS1230/scenefiles/test_unit/unit_cube.xml",
        ),
        Path::new("/Users/smorris/Desktop/Sophomore Year/CS1230/scenefiles/"),
    )
    .unwrap();

    println!("{:#?}", scene);
}

struct Intersection {
    component_intersection: ComponentIntersection,
    material: Material,

}

struct ComponentIntersection {
    t: f32,
    normal: glm::Vec4,
    uv: (f32, f32),
}


trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

struct Shape {
    components: Vec<ShapeComponent>,
    material: Material,
    ctm: glm::Mat4
}

impl Shape {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let mut intersections: Vec<ComponentIntersection> = Vec::new();
        for component in self.components {
            let object_intersection: Option<ComponentIntersection> = component.intersect(ray.toObjectSpace(self.ctm));

            if let Some(intersection) = object_intersection {
                intersections.push(intersection);
            }    
        }

        let closest = intersections.smallestT()
        Some(Intersection {
            closest, 
            self.material 
            // with material too
        })
    }
}

trait ShapeComponent {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection>;
}

// normal_calculator:
//     uv_mapper:
//     solver: Square + Circle + ConeBody + CylinderBody + Sphere

struct Square {
    plane: Plane
}

impl ShapeComponent for Square {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let intersection = self.plane.intersect(ray);

        // Check square constraints
        
        return Some(ComponentIntersection { t, normal, uv })
    }
}

impl ShapeComponent for Circle {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        let (t, normal, uv) = self.plane.intersect(ray);

        // Check circle constraints
        
        return ComponentIntersection { t, normal, uv }
    }
}

impl<T: QuadraticBody> ShapeComponent for T {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        // calculate the a, b, c
        let quadratic = self.calculate_quadratic(ray);
        let solution = solve_quadratic(quadratic).iter().filter(|t| self.check_constraint(ray.at(t))).min()?;
        Some(ComponentIntersection {
            normal: self.get_normal(ray,),
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

struct Plane {
    normal: glm::Vec3,
    elevation: f32,
}

impl Plane {
    fn intersect(&self, ray: &Ray) -> Option<ComponentIntersection> {
        todo!()
    }
}
