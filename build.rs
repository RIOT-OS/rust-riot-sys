extern crate bindgen;
extern crate shlex;

use bindgen::builder;
use std::env;
use std::path::PathBuf;

fn main() {
    let sourcefile = env::var("RIOT_EXPANDED_HEADER")
        .expect("Please set RIOT_EXPANDED_HEADER, see README for details.");

    let cflags = env::var("CFLAGS")
        .expect("Please pass in the same CFLAGS that were used to build RIOT.");
    let cflags = shlex::split(&cflags)
        .expect("Odd shell escaping in CFLAGS");

    println!("cargo:rerun-if-env-changed=RIOT_EXPANDED_HEADER");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-changed={}", sourcefile);

    let bindings = builder()
        .header(sourcefile)
        .clang_args(cflags.iter().filter(|x| {
            match x.as_ref() {
                "-Werror" => false,
                "-mno-thumb-interwork" => false,
                "-Wformat-overflow" => false,
                "-Wformat-truncation" => false,
                _ => true,
            }
        }))
        .use_core()
        .ctypes_prefix("libc")
        .impl_debug(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
