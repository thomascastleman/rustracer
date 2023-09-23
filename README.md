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

Run from the root of the repository, the following command will run a release binary
to render a 1024 by 768 image, using shadows, reflections, texture mapping, parallelism,
and stochastic supersampling:

```
cargo run --release -- --width 1024 --height 768 \
    --output ./output/test_efficiency_recursiveSpheres3.png \
    --textures ./tests/textures \
    --scene ./tests/scenefiles/test_efficiency/recursiveSpheres3.xml \
    --enable-shadows \
    --enable-reflections \
    --enable-texture \
    --enable-parallelism \
    --samples 20
```

## Tests

To run the tests (which will compare rendered output with benchmark images and fail if
significant difference is detected), run:

```
tests/clear_diffs.sh && cargo test
```

The `clear_diffs.sh` script ensures that there are no left-over diff images from a previous test run.

If you add or remove a test scenefile/benchmark, make sure to run

```
tests/generate_test_cases.sh >tests/test_against_benchmarks.rs
```

to update the auto-generated list of macro invocations that generate the test functions for each scenefile/image.

## Documentation

To build the documentation and open it in your browser, run

```
cargo doc --open
```

## Sample Output

For examples of images produced by this program, see the [`output` directory](output).

## Scenefiles

The scenefiles are expected to be in the XML format used by CS1230. Several examples can be found in
the `tests/scenefiles` directory of this repository, or in [this repository](https://github.com/BrownCSCI1230/scenefiles),
where they were adapted from.
