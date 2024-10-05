use std::env;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/nodeMessage.proto")?;

    println!("cargo:rustc-link-search=native=/cuda_mining_lib/include/mining.h");
    println!("cargo:rustc-link-lib=static=mining_lib");

    let cuda_include_path = "/usr/local/cuda/include";
    // Compile the C++/CUDA code using the `cc` crate
    cc::Build::new()
        .cuda(true)
        .file("./cuda_mining_lib/src/mining.cu")
        .include(cuda_include_path) // Add the CUDA include path to the build
        .compile("mining_lib");

    let bindings = bindgen::Builder::default()
        .header("./cuda_mining_lib/include/mining.h")
        .clang_arg("-x") // Specify that this is C++
        .clang_arg("c++")
        .clang_arg(format!("-I{}", cuda_include_path))
        // Tell cargo to invalidate the built crate whenever the wrapper changes
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-lib=dylib=cudart");
    Ok(())
}
