mod scene;

fn main() {
    let scene = scene::Scene::parse(
        "/Users/smorris/Desktop/Sophomore Year/CS1230/scenefiles/test_efficiency/recursiveCones4.xml",
    )
    .unwrap();
}
