use std::path::{Path, PathBuf};

const COMPONENT_FILE: &str = "component.kdl";

fn main() {
    let project = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).parent().unwrap().to_owned();
    let components_folder = project.join("components");
    let web_folder = project.join("web");
    let content_folder = project.join("content");
    let components_out = content_folder.join("components");

    println!("cargo:rerun-if-changed={}/*", web_folder.display());
    println!("cargo:rerun-if-changed={}/*", components_folder.display());

    for entry in std::fs::read_dir(&components_folder).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "lib" {
            continue;
        }
        process_component(&entry.path(), &components_out.join(entry.file_name()));
    }

    let release = !cfg!(debug_assertions);

    for entry in std::fs::read_dir(web_folder).unwrap() {
        let entry = entry.unwrap();
        let outdir = content_folder.join("public").join(entry.file_name());
        build_ts(release, true, &entry.path(), &outdir, &["common", "components"]);
    }
}

fn build_ts(release: bool, subdir: bool, path: &Path, out_path: &Path, exclude: &[&str]) {
    if exclude.contains(&path.file_name().unwrap().to_str().unwrap()) {
        return;
    }

    let bun = std::process::Command::new("bun").args(["--version"]).output().unwrap().status.success();
    if !bun {
        panic!("no bun found");
    }

    let build_name = if release { "build-prod" } else { "build" };

    let out = std::process::Command::new("bun")
        .args(["run", build_name])
        .current_dir(path)
        .output()
        .unwrap();

    if !out.status.success() {
        println!("cargo:warning=bun stderr: {}", String::from_utf8(out.stderr).unwrap());
        println!("cargo:warning=bun stdout: {}", String::from_utf8(out.stdout).unwrap());
        panic!("failed to build {}", path.display());
    }

    let dest_folder = path.join("out");
    let js_out_path = if subdir { out_path.join("js") } else { out_path.to_path_buf() };

    std::fs::create_dir_all(&js_out_path).unwrap();

    let js_file = dest_folder.join("index.js");
    let dest_file = js_out_path.join("index.js");
    std::fs::copy(&js_file, &dest_file).unwrap();

    let css_file = dest_folder.join("index.css");
    if css_file.exists() {
        let dest_folder = if subdir { out_path.join("css") } else { out_path.to_path_buf() };
        let dest_file = dest_folder.join("index.css");
        std::fs::create_dir_all(&dest_folder).unwrap();
        std::fs::copy(&css_file, &dest_file).unwrap();
    }

    let meta_file = path.join("meta.ron");
    if meta_file.exists() {
        let dest_file = out_path.join("meta.ron");
        std::fs::copy(&meta_file, &dest_file).unwrap();
    }
}

fn process_component(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();

    let component = from.join(COMPONENT_FILE);
    if !component.exists() {
        println!("cargo:warning=component.kdl not found in {}", from.display());
        return;
    }

    std::fs::copy(component, to.join(COMPONENT_FILE)).expect("failed to copy component file");
}