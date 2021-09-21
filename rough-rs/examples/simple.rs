use std::{
    fs,
    io::Write,
};

use rough_rs::{generator::RoughGenerator, node_to_string};
use svg::Document;
extern crate nalgebra as na;

fn main() {
    /*     let data = Data::new()
        .move_to((10.0, 10.0))
        .line_by((0.0, 10.0))
        .line_by((10.0, 10.0))
        .close();

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("d", data); */
    let mut rough_generator = RoughGenerator::new(None);

    let mut document = Document::new().set("viewBox", (0, 0, 600, 600));
    for _i in 0..=1 {
        let path_top = rough_generator.line(na::vector![100.0, 100.0], na::vector![100.0, 400.0]);
        let path_bottom = rough_generator.line(na::vector![100.0, 400.0], na::vector![400.0, 400.0]);
        let path_left = rough_generator.line(na::vector![100.0, 100.0], na::vector![400.0, 100.0]);
        let path_right = rough_generator.line(na::vector![400.0, 100.0], na::vector![400.0, 400.0]);

        document = document
            .add(path_top)
            .add(path_bottom)
            .add(path_left)
            .add(path_right);
    }

    let svg = node_to_string(&document).expect("failed to write node as String");

    let mut f = fs::File::create("./examples/simple.svg").expect("Unable to create file");
    f.write_all(svg.as_bytes()).expect("Unable to write data");
}
