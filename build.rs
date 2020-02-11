extern crate bindgen;
extern crate shlex;

use bindgen::builder;
use std::env;
use std::path::PathBuf;

fn main() {
    let cflags = env::var("RIOT_CFLAGS")
        .expect("Please pass in RIOT_CFLAGS; see README.md for details.");
    let cflags = shlex::split(&cflags)
        .expect("Odd shell escaping in RIOT_CFLAGS");

    println!("cargo:rerun-if-env-changed=RIOT_CFLAGS");
    println!("cargo:rerun-if-changed=riot-all.h");

    let bindings = builder()
        .header("riot-all.h")
        .size_t_is_usize(true)
        .clang_args(cflags.iter().filter(|x| {
            match x.as_ref() {
                // non-clang flags showing up with arm cortex m3 (eg. stk3700 board)
                "-Werror" => false,
                "-mno-thumb-interwork" => false,
                "-Wformat-overflow" => false,
                "-Wformat-truncation" => false,
                // non-clang flags showing up for the hifive1 board
                "-mcmodel=medlow" => false,
                "-msmall-data-limit=8" => false,
                "-nostartfiles" => false, // that probably shows up on arm too, but shouldn't matter
                // accept all others
                _ => true,
            }
        }))
        .use_core()
        .ctypes_prefix("libc")
        .impl_debug(true)
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
