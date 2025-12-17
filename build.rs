use std::path::PathBuf;
use std::fs;
use std::env;

fn main() {
    println!("cargo::rerun-if-changed=.");
    watch_dir("./kotlin".into());
    watch_dir("./res".into());

    let build_type = env::var("PROFILE").unwrap();

    let dest = PathBuf::from(format!("./target/dx/trackfish/{build_type}/android/app/app/src/main/kotlin/dev/dioxus/main"));

    fs::create_dir_all(&dest).unwrap();
    for entry in fs::read_dir("./kotlin/").unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), dest.join(entry.file_name())).unwrap();
    }

    let res_dest = PathBuf::from(&format!("./target/dx/trackfish/{build_type}/android/app/app/src/main/res"));
    copy_dir_recursive(PathBuf::from("./res"), res_dest);
}

fn watch_dir(path: PathBuf) {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.is_dir() {
                watch_dir(p);
            } else {
                println!("cargo::rerun-if-changed={}", p.display());
            }
        }
    }
}

fn copy_dir_recursive(src: PathBuf, dst: PathBuf) {
    if src.is_dir() {
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let src_path = entry.path();
            let dest_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                fs::create_dir_all(dest_path.clone()).unwrap();
                copy_dir_recursive(src_path, dest_path);
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::copy(src_path, dest_path).unwrap();
            }
        }
    }
}
