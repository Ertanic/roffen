use std::{env, fs};
use std::path::PathBuf;

fn main() {
    let target = PathBuf::from(env::var("CARGO_BUILD_BUILD_DIR").unwrap());
    let content = target.join("content");
    fs::remove_dir_all(content)
}