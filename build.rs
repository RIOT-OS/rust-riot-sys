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

    #[cfg(not(feature = "riot-rs"))]
    let compile_commands_json = "RIOT_COMPILE_COMMANDS_JSON";
    #[cfg(feature = "riot-rs")]
    let compile_commands_json = "DEP_RIOT_BUILD_COMPILE_COMMANDS_JSON";

    println!("cargo:rerun-if-env-changed=RIOT_CC");
    println!("cargo:rerun-if-env-changed=RIOT_CFLAGS");
    println!("cargo:rerun-if-env-changed={}", &compile_commands_json);

    if let Ok(commands_json) = env::var(compile_commands_json) {
        println!("cargo:rerun-if-changed={}", commands_json);
        let commands_file = std::fs::File::open(&commands_json)
            .expect(&format!("Failed to open {}", &commands_json));

        #[derive(Debug, serde::Deserialize)]
        struct Entry {
            arguments: Vec<String>,
        }
        let parsed: Vec<Entry> = serde_json::from_reader(commands_file)
            .expect(&format!("Failed to parse {}", &compile_commands_json));

        // We need to find a consensus list -- otherwise single modules like stdio_uart that
        // defines anything odd for its own purpose can throw things off. (It's not like the actual
        // ABI compatibility should suffer from them, for any flags like enum packing need to be
        // the same systemwide anyway for things to to go very wrong) -- but at any rate, finding
        // some consensus is to some extent necessary here).
        //
        // This is relatively brittle, but still better than the previous approach of just taking
        // the first entry.
        //
        // A good long-term solution might be to take CFLAGS as the build system produces them, but
        // pass them through the LLVMization process of create_compile_commands without actually
        // turning them into compile commands.
        let mut consensus_cc: Option<&str> = None;
        let mut consensus_cflag_groups: Option<Vec<Vec<&str>>> = None;
        for entry in parsed.iter() {
            if let Some(consensus_cc) = consensus_cc.as_ref() {
                assert!(consensus_cc == &entry.arguments[0])
            } else {
                consensus_cc = Some(&entry.arguments[0]);
            }
            let arg_iter = entry.arguments[1..]
                .iter()
                .map(|s| s.as_str())
                // Anything after -c is not CFLAGS but concrete input/output stuff.
                .take_while(|&s| s != "-c" && s != "-MQ");
            // Heuristically grouping them to drop different arguments as whole group
            let mut cflag_groups = vec![];
            for mut arg in arg_iter {
                if arg.starts_with("-I") {
                    // -I arguments are given inconsistently with and without trailing slashes;
                    // removing them keeps them from being pruned from the consensus set
                    arg = arg.trim_end_matches('/');
                }
                if arg.starts_with('-') {
                    cflag_groups.push(vec![arg]);
                } else {
                    cflag_groups
                        .last_mut()
                        .expect("CFLAG options all start with a dash")
                        .push(arg);
                }
            }
            if let Some(consensus_cflag_groups) = consensus_cflag_groups.as_mut() {
                if &cflag_groups != consensus_cflag_groups {
                    // consensus is in a good ordering, so we'll just strip it down
                    *consensus_cflag_groups = consensus_cflag_groups
                        .drain(..)
                        .filter(|i| {
                            let mut keep = cflag_groups.contains(i);
                            // USEMODULE_INCLUDES are sometimes not in all of the entries; see note
                            // on brittleness above.
                            keep |= i[0].starts_with("-I");
                            // Left as multiple lines to ease hooking in with debug statements when
                            // something goes wrong again...
                            keep
                        })
                        .collect();
                }
            } else {
                consensus_cflag_groups = Some(cflag_groups);
            }
        }
        cc = consensus_cc
            .expect("Entries are present in compile_commands.json")
            .to_string();
        cflags = shlex::join(consensus_cflag_groups.unwrap().iter().flatten().map(|s| *s));

        let usemodule = {
            #[cfg(not(feature = "riot-rs"))]
            {
                println!("cargo:rerun-if-env-changed=RIOT_USEMODULE");
                env::var("RIOT_USEMODULE").expect(&format!(
                    "RIOT_USEMODULE is required when {} is given",
                    &compile_commands_json,
                ))
            }
            #[cfg(feature = "riot-rs")]
            {
                println!("cargo:rerun-if-env-changed=DEP_RIOT_BUILD_DIR");
                let riot_builddir =
                    env::var("DEP_RIOT_BUILD_DIR").expect("DEP_RIOT_BUILD_DIR unset?");
                get_riot_var(&riot_builddir, "USEMODULE")
            }
        };

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
                // These will be in riotbuild.h as well, and better there because bindgen emits
                // consts for data from files but not from defines (?)
                x if x.starts_with("-D") => false,
                // Don't pollute the riot-sys source directory -- cargo is run unconditionally
                // in the Makefiles, and this script tracks on its own which files to depend on
                // for rebuilding.
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
    // Store for inspection for markers; see there
    let mut bindgen_output = Vec::<u8>::new();
    bindings
        .write(Box::new(&mut bindgen_output))
        .expect("String writing never fails");
    let bindgen_output = std::str::from_utf8(&bindgen_output).expect("Rust source code is UTF-8");

    // Build a compile_commands.json, and run C2Rust
    //
    // The output is cleared beforehand (for c2rust no-ops when an output file is present), and the
    // input is copied to OUT_DIR as that's the easiest way to get c2rust to put the output file in
    // a different place -- and because some additions are generated anyway.

    let c2rust_infile = "riot-c2rust.h";
    // Follows from c2rust_infile and C2Rust's file name translation scheme
    let c2rust_output = out_path.join("riot_c2rust.rs");
    let headercopy = out_path.join(c2rust_infile);
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

    if cc.find("clang") == None {
        panic!("riot-sys only accepts clang style CFLAGS. RIOT can produce them using the compile_commands tool even when using a non-clang compiler, such as GCC.");
    };

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
    std::fs::File::open(c2rust_output)
        .expect("Failed to open riot_c2rust.rs")
        .read_to_string(&mut rustcode)
        .expect("Failed to read from riot_c2rust.rs");

    rustcode = rustcode.replace("use ::libc;\n", "");

    // C2Rust still generates old-style ASM -- workaround for https://github.com/immunant/c2rust/issues/306
    rustcode = rustcode.replace(" asm!(", " llvm_asm!(");

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_KEEP_EXTERN_TYPES");
    if env::var("CARGO_FEATURE_KEEP_EXTERN_TYPES").is_err() {
        // There's only one `pub type` usually, and that breaks use on stable, and src/inline.rs has a
        // workaround for that
        rustcode = rustcode.replace("\n    pub type __locale_t;", "");
        rustcode = rustcode.replace("\n    pub type _IO_wide_data;", "");
        rustcode = rustcode.replace("\n    pub type _IO_codecvt;", "");
        rustcode = rustcode.replace("\n    pub type _IO_marker;", "");
        rustcode = rustcode.replace("\n    pub type __lock;", "");
    }

    // This only matches when c2rust is built to even export body-less functions
    rustcode = rustcode.replace("    #[no_mangle]\n    fn ", "    #[no_mangle]\n    pub fn ");

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

            // Assigned by CMSIS to the const that is being overridden and thus needs its original
            // "C" type; see also riot-c2rust.h. (Actually using it would cause a linker error
            // anyway).
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
    //
    // If (eg. on some platform but not on others) any function here is not an inline function,
    // that does not hurt; the entry doesn't do anything on these then. (But it is especially
    // valuable, as it ensures that on the *other* platforms it's still available with the same
    // Rust name).
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
        .filter(|s: &String| rustcode.contains(&format!(" {}(", s)))
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

    // Some decisions of downstream crates need to depend on whether some feature is around in
    // RIOT. For many things that's best checked on module level, but some minor items have no
    // module to mark the feature, and checking for versions by numers is not fine-grained enough,
    // so it's easiest to check for concrete strings in the bindgen output.
    //
    // These lead to `MARKER_foo=1` items emitted that are usable as `DEP_RIOT_SYS_MARKER_foo=1` by
    // crates that explicitly `links = "riot-sys"`. They are stable in that they'll only go away in
    // a breaking riot-sys version; downstream users likely stop using them earlier because they
    // sooner or later stop supporting old RIOT versions.
    //
    // These markers are currently checked against bindgen's output, but could query any property
    // that riot-sys has access to. The markers are defined in terms of some change having happened
    // in RIOT; the way they are tested for can change. (In particular, when riot-sys stops
    // supporting an older RIOT version, it can just always emit that marker).
    //
    // Crates building on this should preferably not alter their own APIs depending on these,
    // because that would add extra (and hard-to-track) dimensions to them. If they can, they
    // should provide a unified view and degrade gracefully. (For example, riot-wrappers has the
    // unit `T` of the `phydat_unit_t` in its enum, but converts it to the generic unspecified unit
    // on RIOT versions that don't have the T type yet -- at least for as long as it supports
    // 2022.01).

    let markers = [
        // See https://github.com/RIOT-OS/RIOT/pull/17569, available after 2022.01
        ("pub const UNIT_T:", "phydat_unit_t"),
        // See https://github.com/RIOT-OS/RIOT/pull/17660, available after 2022.01
        ("vfs_iterate_mount_dirs", "vfs_iterate_mount_dirs"),
    ];
    for (needle, name) in markers {
        if bindgen_output.contains(needle) {
            println!("cargo:MARKER_{}=1", name);
        }
    }
}

#[cfg(feature = "riot-rs")]
fn get_riot_var(riot_builddir: &str, var: &str) -> String {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "{} make --no-print-directory -C {} TOOLCHAIN=llvm info-debug-variable-{}",
            "WARNING_EXTERNAL_MODULE_DIRS=0", riot_builddir, var
        ))
        .output()
        .unwrap()
        .stdout;
    String::from_utf8_lossy(output.as_slice()).trim_end().into()
}
