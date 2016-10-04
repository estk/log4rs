extern crate serde_codegen;
extern crate walkdir;

use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new("../src").into_iter() {
        let entry = entry.unwrap();

        if entry.path().extension().map(|e| e != "in").unwrap_or(true) {
            continue;
        }

        let src = entry.path();
        let dst = src.with_file_name(src.file_stem().unwrap());
        serde_codegen::expand(src, dst).unwrap();
    }
}
