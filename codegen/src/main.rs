extern crate syntex;
extern crate serde_codegen;

use std::path::Path;

fn main() {
    let src = Path::new("../src/toml/serde.rs.in");
    let dst = Path::new("../src/toml/serde.rs");

    let mut registry = syntex::Registry::new();
    serde_codegen::register(&mut registry);

    registry.expand("", src, dst).unwrap();
}
