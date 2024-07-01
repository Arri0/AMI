use const_format::formatcp;
use git2::Repository;
use std::{
    env,
    path::{Path, PathBuf},
};

const SFIZZ_REPO_URL: &str = "https://github.com/sfztools/sfizz.git";
const SFIZZ_WRAPPER_PATH: &str = "./external_c_cpp/sfizz_wrapper";
const SFIZZ_FILE1_PATH: &str = "./external_c_cpp/sfizz_wrapper/sfizz/src/sfizz/FilePool.cpp";
const OUTPUT_FILE_NAME: &str = "sfizz_bindings.rs";

const SFIZZ_PATH: &str = formatcp!("{}/{}", SFIZZ_WRAPPER_PATH, "sfizz");
const HEADER_PATH: &str = formatcp!("{}/{}", SFIZZ_PATH, "src/sfizz.h");

fn main() {
    prepare_external_dependencies();
    link_other_libraries();
    add_rerun_dependencies();
    generate_bindings();
}

fn prepare_external_dependencies() {
    clone_sfizz_repo_if_missing();
    build_and_link_sfizz();
}

fn clone_sfizz_repo_if_missing() {
    if !Path::new(SFIZZ_PATH).exists() {
        Repository::clone_recurse(SFIZZ_REPO_URL, SFIZZ_PATH).unwrap();
        remove_sfizz_unwanted_code().ok();
    }
}

fn remove_sfizz_unwanted_code() -> Result<(), std::io::Error> {
    let mut content = std::fs::read_to_string(SFIZZ_FILE1_PATH)?;
    content = content.replace("raiseCurrentThreadPriority();", "");
    std::fs::write(SFIZZ_FILE1_PATH, content)?;
    Ok(())
}

fn build_and_link_sfizz() {
    let dst = cmake::Config::new(SFIZZ_WRAPPER_PATH)
        .define("CMAKE_BUILD_TYPE", "Release")
        .build();
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=sfizz_static_bundled");

    #[cfg(target_env = "msvc")]
    println!("cargo:rustc-link-lib=static=msvcrtd");
}

fn link_other_libraries() {
    #[cfg(unix)]
    {
        println!("cargo:rustc-link-lib=dylib=stdc++fs");
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}

fn add_rerun_dependencies() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", HEADER_PATH);
}

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        .header(HEADER_PATH)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_path.join(OUTPUT_FILE_NAME);
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
