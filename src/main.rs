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
