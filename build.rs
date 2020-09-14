extern crate bindgen;
extern crate shlex;

use std::fmt::Write;
use bindgen::builder;
use std::env;
use std::path::PathBuf;

use serde_json::json;

fn main() {
    let cflags = if let Ok(_) = std::env::var("DOCS_RS") {
        // For docs.rs, we take this exemplary setup
        //
        // Based on the riot-examples coap build for native (aligned with Cargo.toml's
        // default-target for docs.rs), paths switched over to documentation-headers/
        // which is a headers-only full tree copy of RIOT current master branch 8b3d019d (except
        // boards and cpu keeping only native, which makes this managable in size)
        "-DDEVELHELP -Werror -Wall -Wextra -pedantic -g3 -std=gnu11 -m32 -fstack-protector-all -ffunction-sections -fdata-sections -DDEBUG_ASSERT_VERBOSE -DRIOT_APPLICATION=\"coap_demo\" -DBOARD_NATIVE=\"native\" -DRIOT_BOARD=BOARD_NATIVE -DCPU_NATIVE=\"native\" -DRIOT_CPU=CPU_NATIVE -DMCU_NATIVE=\"native\" -DRIOT_MCU=MCU_NATIVE -fno-common -Wall -Wextra -Wmissing-include-dirs -fno-delete-null-pointer-checks -fdiagnostics-color -Wstrict-prototypes -Wold-style-definition -gz -Wformat=2 -Wformat-overflow -Wformat-truncation -DSOCK_HAS_IPV6 -DSOCK_HAS_ASYNC -DSOCK_HAS_ASYNC -DSOCK_HAS_ASYNC_CTX -DRIOT_VERSION=\"2020.10-devel-1278-g8b3d01\" -DMODULE_AUTO_INIT -DMODULE_AUTO_INIT_GNRC_IPV6 -DMODULE_AUTO_INIT_GNRC_IPV6_NIB -DMODULE_AUTO_INIT_GNRC_NETIF -DMODULE_AUTO_INIT_GNRC_PKTBUF -DMODULE_AUTO_INIT_GNRC_UDP -DMODULE_AUTO_INIT_RANDOM -DMODULE_AUTO_INIT_XTIMER -DMODULE_BOARD -DMODULE_CORE -DMODULE_CORE_IDLE_THREAD -DMODULE_CORE_INIT -DMODULE_CORE_MBOX -DMODULE_CORE_MSG -DMODULE_CORE_PANIC -DMODULE_CORE_THREAD_FLAGS -DMODULE_CPU -DMODULE_DIV -DMODULE_EVENT -DMODULE_EVENT_CALLBACK -DMODULE_EVENT_TIMEOUT -DMODULE_EVTIMER -DMODULE_FMT -DMODULE_GCOAP -DMODULE_GNRC -DMODULE_GNRC_ICMPV6 -DMODULE_GNRC_ICMPV6_ECHO -DMODULE_GNRC_IPV6 -DMODULE_GNRC_IPV6_DEFAULT -DMODULE_GNRC_IPV6_HDR -DMODULE_GNRC_IPV6_NIB -DMODULE_GNRC_NDP -DMODULE_GNRC_NETAPI -DMODULE_GNRC_NETAPI_CALLBACKS -DMODULE_GNRC_NETAPI_MBOX -DMODULE_GNRC_NETDEV_DEFAULT -DMODULE_GNRC_NETIF -DMODULE_GNRC_NETIF_ETHERNET -DMODULE_GNRC_NETIF_HDR -DMODULE_GNRC_NETIF_INIT_DEVS -DMODULE_GNRC_NETIF_IPV6 -DMODULE_GNRC_NETREG -DMODULE_GNRC_NETTYPE_ICMPV6 -DMODULE_GNRC_NETTYPE_IPV6 -DMODULE_GNRC_NETTYPE_UDP -DMODULE_GNRC_PKT -DMODULE_GNRC_PKTBUF -DMODULE_GNRC_PKTBUF_STATIC -DMODULE_GNRC_SOCK -DMODULE_GNRC_SOCK_ASYNC -DMODULE_GNRC_SOCK_UDP -DMODULE_GNRC_UDP -DMODULE_ICMPV6 -DMODULE_INET_CSUM -DMODULE_IOLIST -DMODULE_IPV6_ADDR -DMODULE_IPV6_HDR -DMODULE_L2UTIL -DMODULE_LUID -DMODULE_NANOCOAP -DMODULE_NATIVE_DRIVERS -DMODULE_NETDEV_DEFAULT -DMODULE_NETDEV_ETH -DMODULE_NETDEV_REGISTER -DMODULE_NETDEV_TAP -DMODULE_NETIF -DMODULE_PERIPH -DMODULE_PERIPH_COMMON -DMODULE_PERIPH_CPUID -DMODULE_PERIPH_GPIO -DMODULE_PERIPH_GPIO_LINUX -DMODULE_PERIPH_HWRNG -DMODULE_PERIPH_INIT -DMODULE_PERIPH_INIT_CPUID -DMODULE_PERIPH_INIT_GPIO -DMODULE_PERIPH_INIT_GPIO_LINUX -DMODULE_PERIPH_INIT_HWRNG -DMODULE_PERIPH_INIT_PM -DMODULE_PERIPH_INIT_TIMER -DMODULE_PERIPH_INIT_UART -DMODULE_PERIPH_PM -DMODULE_PERIPH_TIMER -DMODULE_PERIPH_UART -DMODULE_POSIX_HEADERS -DMODULE_POSIX_INET -DMODULE_PRNG -DMODULE_PRNG_TINYMT32 -DMODULE_PS -DMODULE_RANDOM -DMODULE_SHELL -DMODULE_SHELL_COMMANDS -DMODULE_SOCK -DMODULE_SOCK_ASYNC -DMODULE_SOCK_ASYNC_EVENT -DMODULE_SOCK_UDP -DMODULE_SOCK_UTIL -DMODULE_STDIN -DMODULE_STDIO_NATIVE -DMODULE_SYS -DMODULE_TINYMT32 -DMODULE_UDP -DMODULE_XTIMER -Idocumentation-headers/core/include -Idocumentation-headers/drivers/include -Idocumentation-headers/sys/include -Idocumentation-headers/boards/native/include -DNATIVE_INCLUDES -Idocumentation-headers/boards/native/include/ -Idocumentation-headers/core/include/ -Idocumentation-headers/drivers/include/ -Idocumentation-headers/cpu/native/include -Idocumentation-headers/sys/include -Idocumentation-headers/cpu/native/include -Idocumentation-headers/sys/net/gnrc/sock/include -Idocumentation-headers/sys/posix/include -Idocumentation-headers/sys/net/sock/async/event ".to_string()
    } else {
        env::var("RIOT_CFLAGS")
            .expect("Please pass in RIOT_CFLAGS; see README.md for details.")
            .clone()
    };
    let cflags = shlex::split(&cflags)
        .expect("Odd shell escaping in RIOT_CFLAGS");

    println!("cargo:rerun-if-env-changed=RIOT_CFLAGS");
    println!("cargo:rerun-if-changed=riot-bindgen.h");

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
        .header("riot-bindgen.h")
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
        write!(c_code, r"

static {type_name} init_{macro_name}(void) {{
    {type_name} result = {macro_name};
    return result;
}}
            ", type_name=type_name, macro_name=macro_name,
            ).unwrap();
    }

    let mut outfile = std::fs::File::create(headercopy)
        .expect("Failed to open temporary riot-c2rust.h");
    outfile
        .write_all(c_code.as_bytes())
        .expect("Failed to write to riot-c2rust.h");
    outfile
        .sync_all()
        .expect("failed to write to riot-c2rust.h");

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
    // FIXME: This does not rat on the used files. Most are probably included from riot-bindgen.h
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
    rustcode = rustcode.replace(r#"unsafe extern "C" fn "#, r#"pub unsafe fn "#);
    // used as a callback, therefore does need the extern "C" -- FIXME probably worth a RIOT issue
    rustcode = rustcode.replace(r"pub unsafe fn _evtimer_msg_handler", r#"pub unsafe extern "C" fn _evtimer_msg_handler"#);
    // particular functions known to be const because they have macro equivalents as well
    // (Probably we could remove the 'extern "C"' from all functions)
    rustcode = rustcode.replace(r#"pub unsafe extern "C" fn mutex_init("#, r#"pub const unsafe fn mutex_init("#);

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
