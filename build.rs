use std::path::PathBuf;
use std::{io, fs};
use std::env;

fn main() {
    for entry in fs::read_dir("./kotlin").unwrap() {
        let entry = entry.unwrap();
        println!("cargo::rerun-if-changed={}", entry.path().display());
    }

    let dest = PathBuf::from(format!("./target/dx/trackfish/release/android/app/app/src/main/kotlin/dev/dioxus/main"));

    fs::create_dir_all(&dest).unwrap();
    for entry in fs::read_dir("./kotlin/").unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), dest.join(entry.file_name())).unwrap();
    }
}
