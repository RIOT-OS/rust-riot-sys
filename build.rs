extern crate bindgen;
extern crate shlex;

use bindgen::builder;
use std::env;
use std::fmt::Write;
use std::path::PathBuf;

use serde_json::json;

fn main() {
    let cc;
    let mut cflags;

    if let Ok(commands_json) = env::var("RIOT_COMPILE_COMMANDS_JSON") {
        println!("cargo:rerun-if-env-changed=RIOT_COMPILE_COMMANDS_JSON");
        println!("cargo:rerun-if-changed={}", commands_json);
        let commands_file =
            std::fs::File::open(commands_json).expect("Failed to open RIOT_COMPILE_COMMANDS_JSON");

        #[derive(Debug, serde::Deserialize)]
        struct Entry {
            arguments: Vec<String>,
        }
        let parsed: Vec<Entry> = serde_json::from_reader(commands_file)
            .expect("Failed to parse RIOT_COMPILE_COMMANDS_JSON");

        // Should we only pick the consensus set here?
        let any = &parsed[0];

        cc = any.arguments[0].clone();
        cflags = shlex::join(
            any.arguments[1..]
                .iter()
                .map(|s| s.as_str())
                // Anything after -c is not CFLAGS but concrete input/output stuff
                .take_while(|&s| s != "-c"),
        );

        println!("cargo:rerun-if-env-changed=RIOT_USEMODULE");
        let usemodule = env::var("RIOT_USEMODULE")
            .expect("RIOT_USEMODULE is required when RIOT_COMPILE_COMMANDS_JSON is given");
        for m in usemodule.split(" ") {
            // Hack around https://github.com/RIOT-OS/RIOT/pull/16129#issuecomment-805810090
            write!(
                cflags,
                " -DMODULE_{}",
                m.to_uppercase()
                    // avoid producing MODULE_BOARDS_COMMON_SAMDX1-ARDUINO-BOOTLOADER
                    .replace('-', "_")
            )
            .unwrap();
        }
    } else {
        cc = env::var("RIOT_CC")
            .expect("Please pass in RIOT_CC; see README.md for details.")
            .clone();
        cflags = env::var("RIOT_CFLAGS")
            .expect("Please pass in RIOT_CFLAGS; see README.md for details.");
    }

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
                // Don't pollute the riot-sys source directory
                "-MD" => false,
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
    let macro_functions = [
        ("SOCK_IPV4_EP_ANY", "sock_udp_ep_t", "void", true),
        ("SOCK_IPV6_EP_ANY", "sock_udp_ep_t", "void", true),
        ("MUTEX_INIT", "mutex_t", "void", true),
        // neither C2Rust nor bindgen understand the cast without help
        ("STATUS_NOT_FOUND", "thread_status_t", "void", true),
        // If any board is ever added that works completely differently, this'll have to go behind
        // a feature-gate
        ("GPIO_PIN", "gpio_t", "unsigned port, unsigned pin", true),
    ];
    let mut macro_functions: Vec<_> = macro_functions
        .iter()
        .map(|(macro_name, return_type, args, is_const)| {
            (macro_name.to_string(), *return_type, *args, *is_const)
        })
        .collect();
    for i in 0..8 {
        macro_functions.push((format!("LED{}_ON", i), "void", "void", false));
        macro_functions.push((format!("LED{}_OFF", i), "void", "void", false));
        macro_functions.push((format!("LED{}_TOGGLE", i), "void", "void", false));
    }

    let mut c_code = String::new();
    std::fs::File::open("riot-c2rust.h")
        .expect("Failed to open riot-c2rust.h")
        .read_to_string(&mut c_code)
        .expect("Failed to read riot-c2rust.h");

    for (macro_name, return_type, args, _is_const) in macro_functions.iter() {
        // The ifdef guards make errors easier to spot: A "cannot find function
        // `macro_SOCK_IPV6_EP_ANY` in crate `riot_sys`" can lead one to check whether
        // SOCK_IPV6_EP_ANY is really defined, whereas if the macro is missing, C2Rust would
        // produce a run-time panic, and the compiler would reject that in a const function.
        //
        // This is more reliable than the previous approach of trying to defined a `-DSOME_MODULE`
        // condition, also because there may not even be a module that gives a precise condition.
        if *return_type == "void" {
            // in C, assigning and returning void is special
            write!(
                c_code,
                r"

#ifdef {macro_name}
{return_type} macro_{macro_name}({args}) {{
    {macro_name};
}}
#endif
                ",
                return_type = return_type,
                macro_name = macro_name,
                args = args,
            )
        } else {
            write!(
                c_code,
                r"

#ifdef {macro_name}
{return_type} macro_{macro_name}({args}) {{
    {return_type} result = {macro_name};
    return result;
}}
#endif
                ",
                return_type = return_type,
                macro_name = macro_name,
                args = args,
            )
        }
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
            "--overwrite-existing",
            "--fail-on-error",
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

    // C2Rust still generates old-style ASM -- workaround for https://github.com/immunant/c2rust/issues/306
    rustcode = rustcode.replace(" asm!(", " llvm_asm!(");

    // This only matches when c2rust is built to even export body-less functions
    rustcode = rustcode.replace("    #[no_mangle]\n    fn ", "    #[no_mangle]\n    pub fn ");

    // The GCLK global constant interferes with arguments and fields named GCLK in bindgen, as
    // these would (due to the `use inline::*;` blanket import) try to make that into a matchable
    // pattern. Renaming the whole constant.
    //
    // This is needed, for example, on the adafruit-itsybitsy-m4 board with the riot-hello-world
    // example.
    //
    // Likewise, the SERCOM constants are set by the samr30-xpro board.
    let gclk_word = regex::Regex::new(r"\bGCLK\b").expect("Invalid static regular expression");
    rustcode = gclk_word.replace_all(&rustcode, "GLOBAL_GCLK").to_string();
    let sercom_word =
        regex::Regex::new(r"\bSERCOM[0-9]\b").expect("Invalid static regular expression");
    rustcode = sercom_word.replace_all(&rustcode, "GLOBAL_$0").to_string();

    // Replace the function declarations with ... usually something pub, but special considerations
    // may apply
    let mut rustcode_functionsreplaced = String::new();
    let function_original_prefix = r#"unsafe extern "C" fn "#;
    let mut functionchunks = rustcode.split(function_original_prefix);
    rustcode_functionsreplaced.push_str(
        functionchunks
            .next()
            .expect("Split produces at least a hit"),
    );

    for chunk in functionchunks {
        let funcname = &chunk[..chunk.find('(').expect("Function has parentheses somewhere")];
        let macro_details = if funcname.len() > 5 && &funcname[..6] == "macro_" {
            macro_functions
                .iter()
                .filter(|(macro_name, _, _, _)| &funcname[6..] == *macro_name)
                .next()
        } else {
            None
        };
        let new_prefix = match (funcname, macro_details) {
            // used as a callback, therefore does need the extern "C" -- FIXME probably worth a RIOT issue
            ("_evtimer_msg_handler" | "_evtimer_mbox_handler", _) => function_original_prefix,

            // Assigned by CMSIS to some const; see also riot-c2rust.h
            ("__masked_builtin_arm_get_fpscr" | "__masked_builtin_arm_set_fpscr", _) => {
                function_original_prefix
            }

            // same problem but from C2Rust's --translate-const-macros
            ("__NVIC_SetPriority", _) => function_original_prefix,

            // As below (no need for extern), and they are const as declared ni the macro_functions
            // list.
            (_, Some((_, _, _, is_const))) => {
                // No "pub" because that's already a "pub" in front of it, they were never static
                match is_const {
                    // FIXME: These should be unsafe -- just because most of them are const doesn't
                    // necessrily mean they're safe (just the first few happened to be, but that's
                    // not this crate's place to assert)
                    true => "const unsafe fn ",
                    false => "unsafe fn ",
                }
            }

            // C2Rust transpiles these into Rust with conflicting lifetimes, see
            // https://github.com/immunant/c2rust/issues/309
            //
            // Simply disabling them here because they aren't used by any other inline code (and
            // will, when the manual llvm_asm to asm changes are added to riot-sys, not have manual
            // asm conversions on top of that).
            ("__SMLALD" | "__SMLALDX" | "__SMLSLD" | "__SMLSLDX", _) => {
                "#[cfg(c2rust_fixed_309)]\npub unsafe fn "
            }

            // The rest we don't need to call through the extern convention, but let's please make
            // them pub to be usable
            _ => "pub unsafe fn ",
        };
        rustcode_functionsreplaced.push_str(new_prefix);
        rustcode_functionsreplaced.push_str(chunk);
    }

    rustcode = rustcode_functionsreplaced;

    let output_replaced = out_path.join("riot_c2rust_replaced.rs");
    std::fs::File::create(output_replaced)
        .expect("Failed to create riot_c2rust_replaced.rs")
        .write(rustcode.as_bytes())
        .expect("Failed to write to riot_c2rust_replaced.rs");

    // Pub uses of inline right into the main lib.rs
    //
    // This is primarily for things that can really come from either backend (eg. irq functions
    // that are regular on native but static inline on others), and for convenience stuff like
    // macro_.
    //
    // Some functions are also in because they're innocuous enough.
    let mut toplevel_from_inline: Vec<String> = [
        "bluetil_ad_add_flags",
        "coap_get_code_raw",
        "coap_get_total_hdr_len",
        "gnrc_netapi_dispatch_send",
        "gnrc_netif_ipv6_addrs_get",
        "gnrc_netreg_entry_init_pid",
        "gpio_is_valid",
        "irq_disable",
        "irq_is_enabled",
        "irq_is_in",
        "irq_restore",
        "mutex_trylock",
        "pid_is_valid",
        "shell_run_forever",
        "sock_udp_recv",
        "sock_udp_send",
        "thread_get",
        "thread_getpid",
        "thread_get_unchecked",
        "ztimer_spin",
    ]
    .iter()
    .map(|name| name.to_string())
    .collect();
    for (macro_name, _, _, _) in macro_functions.iter() {
        toplevel_from_inline.push(format!("macro_{}", macro_name));
    }
    let toplevel_from_inline: Vec<String> = toplevel_from_inline
        .drain(..)
        .filter(|s: &String| rustcode.contains(s))
        .collect();
    let toplevel_from_inline_filename = out_path.join("toplevel_from_inline.rs");
    std::fs::File::create(toplevel_from_inline_filename)
        .expect("Failed to create toplevel_from_inline.rs")
        .write(
            format!(
                "
               pub use inline::{{ {} }};
           ",
                toplevel_from_inline.join(",\n")
            )
            .as_bytes(),
        )
        .expect("Failed to write to toplevel_from_inline.rs");
}
