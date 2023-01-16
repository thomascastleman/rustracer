use std::path::Path;

mod intersection;
mod scene;
mod shapes;

fn main() {
    let scene = scene::TreeScene::parse(
        Path::new(
            "/Users/smorris/Desktop/Sophomore Year/CS1230/scenefiles/test_unit/unit_cube.xml",
        ),
        Path::new("/Users/smorris/Desktop/Sophomore Year/CS1230/scenefiles/"),
    )
    .unwrap();

    println!("{:#?}", scene);
}
