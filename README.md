# rustracer

![](output/take_forever_recursiveSpheres3.png)

Offline rendering of 3D graphics using raytracing, written in Rust, and inspired by [Brown's CS1230](https://cs.brown.edu/courses/csci1230).

Written by [Stewart Morris](https://github.com/stew2003) & [Thomas Castleman](https://github.com/thomascastleman/).

## Usage

To build, run

```
cargo build --release
```

The binary can then be found at `target/release/rustracer`. Use the `--help` flag to get a full description of the command line interface.

### Example usage

The following command will run a release binary to render a 1024 by 768 image, using shadows, reflections, texture mapping, parallelism, and stochastic supersampling (change the paths to fit your system):

```
cargo run --release -- --width 1024 --height 768 \
    --output ./output/take_forever_recursiveSpheres3.png \
    --textures ~/Desktop/courses/1230/scenefiles \
    --scene ~/Desktop/courses/1230/scenefiles/test_take_forever/recursiveSpheres3.xml \
    --enable-shadows \
    --enable-reflections \
    --enable-texture \
    --enable-parallelism \
    --samples 20
```

## Documentation

To build the documentation and open it in your browser, run

```
cargo doc --open
```

## Sample Output

For examples of images produced by this program, see the [`output` directory](output).

## Scenefiles

The scenefiles are expected to be in the XML format used by CS1230. [Several examples can be found in this repository](https://github.com/BrownCSCI1230/scenefiles).
