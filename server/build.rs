use std::path::PathBuf;

fn main() {
    build_ts(!cfg!(debug_assertions));
}

fn build_ts(release: bool) {
    let bun = std::process::Command::new("bun").args(["--version"]).output().unwrap().status.success();
    if !bun {
        panic!("no bun found");
    }

    let project = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).parent().unwrap().to_owned();

    // editor
    let editor_folder = project.join("editor");

    println!("cargo:rerun-if-changed={}/*", editor_folder.display());

    let build_name = if release { "build-prod" } else { "build" };

    std::process::Command::new("bun")
        .args(["run", build_name])
        .current_dir(&editor_folder)
        .output()
        .unwrap();

    let editor_script_output = project.join("content").join("public").join("editor").join("js");
    let editor_script = editor_folder.join("out").join("index.js");

    std::fs::create_dir_all(&editor_script_output).unwrap();
    std::fs::copy(editor_script, editor_script_output.join("editor.js")).unwrap();

    // components
    let components_folder = project.join("components");
    let dest_folder = project.join("content").join("components");

    if dest_folder.exists() {
        std::fs::remove_dir_all(&dest_folder).unwrap();
    }

    for entry in std::fs::read_dir(components_folder).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            panic!("{} is not a directory", path.display());
        }

        println!("cargo:rerun-if-changed={}/*", path.display());

        let out_folder = path.join("out");
        if out_folder.exists() {
            std::fs::remove_dir_all(&out_folder).unwrap();
        }

        if std::process::Command::new("bun")
            .args(["run", build_name])
            .current_dir(&path)
            .output()
            .is_err()
        {
            std::process::Command::new("bun")
                .args(["build", "--outdir", "out", "./index.ts", if release { "--production" } else { "" }])
                .current_dir(&path)
                .output()
                .unwrap();
        }

        let dest_folder = dest_folder.join(path.file_name().unwrap());
        std::fs::create_dir_all(&dest_folder).unwrap();

        let meta_path = path.join("meta.ron");
        let dest_meta_path = dest_folder.join("meta.ron");
        std::fs::copy(meta_path, dest_meta_path).expect("failed to copy meta.ron");

        let out_file = out_folder.join("index.js");
        let dest_file = dest_folder.join("index.js");
        std::fs::copy(out_file, dest_file).expect("failed to copy index.js");
    }

    // main
    let main_project = project.join("main");
    println!("cargo:rerun-if-changed={}/*", main_project.display());
    std::process::Command::new("bun")
        .args(["run", build_name])
        .current_dir(&main_project)
        .output()
        .unwrap();

    let main_output = project.join("content").join("public").join("main").join("js");
    let main_script = main_project.join("out").join("index.js");
    std::fs::create_dir_all(&main_output).unwrap();
    std::fs::copy(main_script, main_output.join("index.js")).unwrap();

    let main_css_output = project.join("content").join("public").join("main").join("css");
    let main_css = main_project.join("out").join("index.css");
    std::fs::create_dir_all(&main_css_output).unwrap();
    std::fs::copy(main_css, main_css_output.join("index.css")).unwrap();

    // view
    let view_project = project.join("view");
    println!("cargo:rerun-if-changed={}/*", view_project.display());
    std::process::Command::new("bun")
        .args(["run", build_name])
        .current_dir(&view_project)
        .output()
        .unwrap();
    
    let view_output = project.join("content").join("public").join("view").join("js");
    let view_script = view_project.join("out").join("index.js");
    std::fs::create_dir_all(&view_output).unwrap();
    std::fs::copy(view_script, view_output.join("index.js")).unwrap();
}
