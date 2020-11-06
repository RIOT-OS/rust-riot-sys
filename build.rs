extern crate bindgen;
extern crate shlex;

use bindgen::builder;
use std::env;
use std::fmt::Write;
use std::path::PathBuf;

use serde_json::json;

fn main() {
    let cc = env::var("RIOT_CC")
        .expect("Please pass in RIOT_CC; see README.md for details.")
        .clone();
    let cflags =
        env::var("RIOT_CFLAGS").expect("Please pass in RIOT_CFLAGS; see README.md for details.");

    println!("cargo:rerun-if-env-changed=RIOT_CC");
    println!("cargo:rerun-if-env-changed=RIOT_CFLAGS");

    // pass CC and CFLAGS to dependees
    // this requires a `links = "riot-sys"` directive in Cargo.toml.
    // Dependees can then access these as DEP_RIOT_SYS_CC and DEP_RIOT_SYS_CFLAGS.
    println!("cargo:CC={}", &cc);
    println!("cargo:CFLAGS={}", &cflags);

    println!("cargo:rerun-if-changed=riot-bindgen.h");

    let cflags = shlex::split(&cflags).expect("Odd shell escaping in RIOT_CFLAGS");
    let cflags: Vec<String> = cflags
        .into_iter()
        .filter(|x| {
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
        })
        .collect();

    let bindings = builder()
        .header("riot-bindgen.h")
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
    println!("cargo:rerun-if-changed=riot-c2rust.h");

    std::fs::copy("riot-headers.h", out_path.join("riot-headers.h"))
        .expect("Failed to copy over header file");

    // These constant initializers are unusable without knowledge of which type they're for; adding
    // the information here to build explicit consts
    let struct_initializers = [
        ("SOCK_IPV4_EP_ANY", "sock_udp_ep_t", Some("-DMODULE_SOCK")),
        ("SOCK_IPV6_EP_ANY", "sock_udp_ep_t", Some("-DMODULE_SOCK")),
        ("MUTEX_INIT", "mutex_t", None),
    ];

    let mut c_code = String::new();
    std::fs::File::open("riot-c2rust.h")
        .expect("Failed to open riot-c2rust.h")
        .read_to_string(&mut c_code)
        .expect("Failed to read riot-c2rust.h");

    for (macro_name, type_name, condition) in struct_initializers.iter() {
        if let Some(required_module) = condition {
            if cflags.iter().find(|i| i == required_module).is_none() {
                continue;
            }
        }
        write!(
            c_code,
            r"

static {type_name} init_{macro_name}(void) {{
    {type_name} result = {macro_name};
    return result;
}}
            ",
            type_name = type_name,
            macro_name = macro_name,
        )
        .unwrap();
    }

    let mut outfile =
        std::fs::File::create(&headercopy).expect("Failed to open temporary riot-c2rust.h");
    outfile
        .write_all(c_code.as_bytes())
        .expect("Failed to write to riot-c2rust.h");
    outfile
        .sync_all()
        .expect("failed to write to riot-c2rust.h");

    let c2rust_infile;
    let c2rust_outfile;
    if cc.find("clang") == None {
        // Run through preprocessor with platform specific arguments (cf.
        // <https://github.com/immunant/c2rust/issues/305>)
        //
        // This is only done for non-clang setups; those do not need it (and can profit from the
        // unexpanded macros). Also, clang does not have "-fdirectives-only' (but their
        // "-frewrite-includes" might do as well if it turns out that this *is* needed even there).
        let preprocessed_headercopy = out_path.join("riot-c2rust-expanded.h");
        let clang_e_args: Vec<_> = cflags
            .iter()
            .map(|s| s.clone())
            .chain(
                vec![
                    "-E",
                    "-fdirectives-only",
                    headercopy.to_str().expect("Non-string path for headercopy"),
                    "-o",
                    preprocessed_headercopy
                        .to_str()
                        .expect("Non-string path in preprocessed_headercopy"),
                ]
                .drain(..)
                .map(|x| x.to_string()),
            )
            .collect();
        let status = std::process::Command::new(cc)
            .args(clang_e_args)
            .status()
            .expect("Preprocessor run failed");
        if !status.success() {
            println!(
                "cargo:warning=Preprocessor failed with error code {}, exiting",
                status
            );
            std::process::exit(status.code().unwrap_or(1));
        }
        c2rust_infile = "riot-c2rust-expanded.h";
        c2rust_outfile = "riot_c2rust_expanded.rs";
    } else {
        c2rust_infile = "riot-c2rust.h";
        c2rust_outfile = "riot_c2rust.rs";
    }

    let output = out_path.join(c2rust_outfile);
    match std::fs::remove_file(&output) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => panic!("Failed to remove output file: {}", e),
    }

    let arguments: Vec<_> = core::iter::once("any-cc".to_string())
        .chain(cflags.into_iter())
        .chain(core::iter::once(c2rust_infile.to_string()))
        .collect();
    let compile_commands = json!([{
        "arguments": arguments,
        "directory": out_path,
        "file": c2rust_infile,
    }]);
    let compile_commands_name = out_path.join("compile_commands.json");

    let mut compile_commands_file = std::fs::File::create(compile_commands_name.clone())
        .expect("Failed to create compile_commands.json");
    serde_json::to_writer_pretty(&mut compile_commands_file, &compile_commands)
        .expect("Failed to write to compile_commands.json");
    compile_commands_file
        .sync_all()
        .expect("Failed to write to compile_commands.json");

    let compile_commands_name = compile_commands_name
        .to_str()
        .expect("Inexpressible path name");
    // FIXME: This does not rat on the used files. Most are probably included from riot-bindgen.h
    // anyway, tough.
    println!("Running C2Rust on {}", compile_commands_name);
    let status = std::process::Command::new("c2rust")
        .args(&[
            "transpile",
            compile_commands_name,
            "--preserve-unused-functions",
            "--emit-modules",
            "--emit-no-std",
            "--translate-const-macros",
        ])
        .status()
        .expect("C2Rust failed");
    if !status.success() {
        println!(
            "cargo:warning=C2Rust failed with error code {}, exiting",
            status
        );
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
    rustcode = rustcode.replace(r#"unsafe extern "C" fn "#, r#"pub unsafe fn "#);
    // This only matches when c2rust is built to even export body-less functions
    rustcode = rustcode.replace("    #[no_mangle]\n    fn ", "    #[no_mangle]\n    pub fn ");
    // used as a callback, therefore does need the extern "C" -- FIXME probably worth a RIOT issue
    rustcode = rustcode.replace(
        r"pub unsafe fn _evtimer_msg_handler",
        r#"pub unsafe extern "C" fn _evtimer_msg_handler"#,
    );
    // same problem but from C2Rust's --translate-const-macros
    rustcode = rustcode.replace(
        r"pub unsafe fn __NVIC_SetPriority",
        r#"pub unsafe extern "C" fn __NVIC_SetPriority"#,
    );
    // C2Rust still generates old-style ASM -- workaround for https://github.com/immunant/c2rust/issues/306
    rustcode = rustcode.replace(" asm!(", " llvm_asm!(");
    // particular functions known to be const because they have macro equivalents as well
    // (Probably we could remove the 'extern "C"' from all functions)
    rustcode = rustcode.replace(
        r#"pub unsafe extern "C" fn mutex_init("#,
        r#"pub const unsafe fn mutex_init("#,
    );

    for (macro_name, _, _) in struct_initializers.iter() {
        let search = format!(r#"pub unsafe fn init_{}"#, macro_name);
        let replace = format!(r#"pub const fn init_{}"#, macro_name);
        rustcode = rustcode.replace(&search, &replace);
    }

    let output_replaced = out_path.join("riot_c2rust_replaced.rs");
    std::fs::File::create(output_replaced)
        .expect("Failed to create riot_c2rust_replaced.rs")
        .write(rustcode.as_bytes())
        .expect("Failed to write to riot_c2rust_replaced.rs");
}
