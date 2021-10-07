use std::{fs, io::Write};

use rough_rs::{generator::RoughGenerator, node_to_string};
use svg::Document;
extern crate nalgebra as na;

fn main() {
    let mut document = Document::new().set("viewBox", (0, 0, 500, 500));

    let mut rough_generator = RoughGenerator::new(None);

    // Setting a seed allows for reproducable lines.
    //rough_generator.config.seed = Some(1);

    for _i in 0..=1 {
        let path_top = rough_generator.line(na::vector![100.0, 100.0], na::vector![100.0, 400.0]);
        let path_bottom =
            rough_generator.line(na::vector![100.0, 400.0], na::vector![400.0, 400.0]);
        let path_left = rough_generator.line(na::vector![100.0, 100.0], na::vector![400.0, 100.0]);
        let path_right = rough_generator.line(na::vector![400.0, 100.0], na::vector![400.0, 400.0]);

        document = document
            .add(path_top)
            .add(path_bottom)
            .add(path_left)
            .add(path_right);
    }

    let path_cubic_bezier = rough_generator.cubic_bezier(
        na::vector![20.0, 20.0],
        na::vector![20.0, 200.0],
        na::vector![200.0, 200.0],
        na::vector![400.0, 200.0],
    );
    document = document.add(path_cubic_bezier);

    let svg = node_to_string(&document).expect("failed to write node as String");

    let mut f = fs::File::create("./tests/output/simple.svg").expect("Unable to create file");
    f.write_all(svg.as_bytes()).expect("Unable to write data");
}
