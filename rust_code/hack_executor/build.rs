fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    // println!("cargo:rustc-link-search=../../c_code/hack_executor/build/objs");
    println!("cargo:rustc-link-search=../c_code/hack_executor/build/objs/");

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=hack_memory.o");
    println!("cargo:rustc-link-lib=hack_alu.o");
    println!("cargo:rustc-link-lib=hack_pc.o");
    println!("cargo:rustc-link-lib=hack_executor.o");
}
