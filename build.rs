extern crate bindgen;
extern crate shlex;

use bindgen::builder;
use std::env;
use std::path::PathBuf;

use serde_json::json;

fn main() {
    let cflags = env::var("RIOT_CFLAGS")
        .expect("Please pass in RIOT_CFLAGS; see README.md for details.");
    let cflags = shlex::split(&cflags)
        .expect("Odd shell escaping in RIOT_CFLAGS");

    println!("cargo:rerun-if-env-changed=RIOT_CFLAGS");
    println!("cargo:rerun-if-changed=riot-all.h");

    let cflags: Vec<String> = cflags.into_iter().filter(|x| {
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
            "-fno-delete-null-pointer-checks" => false, // seen on an Ubuntu 18.04
            // and much more worries on that ubuntu ... maybe just recommend TOOLCHAIN=llvm ?
            // accept all others
            _ => true,
        }
    }).collect();

    let bindings = builder()
        .header("riot-all.h")
        .size_t_is_usize(true)
        .clang_args(&cflags)
        .use_core()
        .ctypes_prefix("libc")
        .impl_debug(true)
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Build a compile_commands.json, and run C2Rust
    //
    // The output is cleared beforehand (for c2rust no-ops when an output file is present), and the
    // input is copied to OUT_DIR as that's the easiest way to get c2rust to put the output file in
    // a different place.

    let headercopy = out_path.join("riot-c2rust.h");
    let output = out_path.join("riot_c2rust.rs");
    println!("cargo:rerun-if-changed=riot-c2rust.h");
    std::fs::copy("riot-c2rust.h", headercopy)
        .expect("Failed to copy over header file");
    match std::fs::remove_file(&output) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => panic!("Failed to remove output file: {}", e),
    }

    let arguments: Vec<_> = core::iter::once("any-cc".to_string())
        .chain(cflags.into_iter())
        .chain(core::iter::once("riot-c2rust.h".to_string()))
        .collect();
    let compile_commands = json!([{
        "arguments": arguments,
        "directory": out_path,
        "file": "riot-c2rust.h",
    }]);
    let compile_commands_name = out_path.join("compile_commands.json");

    let mut compile_commands_file = std::fs::File::create(compile_commands_name.clone())
        .expect("Failed to create compile_commands.json");
    serde_json::to_writer_pretty(&mut compile_commands_file, &compile_commands)
        .expect("Failed to write to compile_commands.json");
    compile_commands_file.sync_all()
        .expect("Failed to write to compile_commands.json");

    let compile_commands_name = compile_commands_name.to_str().expect("Inexpressible path name");
    // FIXME: This does not rat on the used files. Most are probably included from riot-all.h
    // anyway, tough.
    println!("Running C2Rust on {}", compile_commands_name);
    let status = std::process::Command::new("c2rust")
        .args(&["transpile", compile_commands_name, "--preserve-unused-functions", "--emit-modules", "--emit-no-std"])
        .status()
        .expect("C2Rust failed");
    if !status.success() {
        println!("cargo:warning=C2Rust failed with error code {}, exiting", status);
        std::process::exit(status.code().unwrap_or(1));
    }

    // Some fix-ups to the C2Rust output
    // (could just as well call sed...)

    use std::io::{Read, Write};

    let mut rustcode = String::new();
    std::fs::File::open(output)
        .expect("Failed to open riot_c2rust.rs")
        .read_to_string(&mut rustcode)
        .expect("Failed to read from riot_c2rust.rs");

    rustcode = rustcode.replace("use ::libc;\n", "");
    rustcode = rustcode.replace(r#"unsafe extern "C" fn "#, r#"pub unsafe extern "C" fn "#);
    // particular functions known to be const because they have macro equivalents as well
    // (Probably we could remove the 'extern "C"' from all functions)
    rustcode = rustcode.replace(r#"pub unsafe extern "C" fn mutex_init("#, r#"pub const unsafe fn mutex_init("#);

    let output_replaced = out_path.join("riot_c2rust_replaced.rs");
    std::fs::File::create(output_replaced)
        .expect("Failed to create riot_c2rust_replaced.rs")
        .write(rustcode.as_bytes())
        .expect("Failed to write to riot_c2rust_replaced.rs");

}
