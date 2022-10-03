//! # Bindings for RIOT system calls
//!
//! This crate contains dynamically generated Rust FFI bindings to the [RIOT
//! Operating System](https://riot-os.org/).
//!
//! Those bindings are inherently unsafe; it is recommended that their safe
//! abstractions in the [riot-wrappers] crate are used in most applications.
//!
//! For a newcomer's starting point, see [RIOT's documentation on using it with Rust].
//!
//! [riot-wrappers]: https://crates.io/crates/riot-wrappers
//! [RIOT's documentation on using it with Rust]: https://doc.riot-os.org/using-rust.html
//!
//! ## RIOT integration
//!
//! Which functions and structs are present in this crate, and sometimes their
//! details, inherently depends on the RIOT configuration this will be used with.
//! For example, RIOT's `thread_t` only has a member `name` if `DEVHELP` is
//! set for a build, and its `flags` member is only present if the `thread_flags`
//! module is in use.
//!
//! All the relevant information -- including the location of the actually used
//! RIOT header files and flags influencing the ABI -- is conveyed to `riot-sys` by passing on the
//! compiler and the CFLAGS. This can either be done by passing in th epath to a "compile commads"
//! file through the `RIOT_COMPILE_COMMANDS` environment variable (accompanied by a
//! `RIOT_USEMODULES`, as that part of `CFLAGS` is missing from the compile commands), or
//! alternatively by passing in the C compiler as `RIOT_CC` and the CFLAGS (both their
//! `CFLAGS_WITH_MACROS` and the `INCLUDES` part from RIOT's build system) in. When called from
//! within RIOT's build system, care must be taken to clear `CC` and `CFLAGS`, as these would be
//! interpreted by Cargo (Rust's build system) to refer to the host compiler and flags.
//! The flags will be interpreted by libclang based tools; care must be taken to pass in flags
//! suitable for clang and not for GCC.
//!
//! These steps are automated in RIOT's build system.
//!
//!
//! The `RIOT_CC` and `RIOT_CFLAGS` are made available to dependent crates through
//! Cargo (as `DEP_RIOT_SYS_CC` etc); see [riot-wrappers]'s build.sh for an example. Similarly,
//! custom markers are made available based on the presence of certain defines or features in RIOT
//! as downstream crates require that information (typically to allow a crate to work across a
//! wider range of RIOT versions); see the comments in `build.rs` for details.
//!
//!
//! ## Extension
//!
//! Currently, only a subset of all the RIOT headers is processed; all the relevant
//! header files are included in this crate's `riot-headers.h` header file. If you
//! need access to more RIOT APIs, more includes can be added there.
//!
//! ## External build dependencies
//!
//! This crate's operation depends on [C2Rust] being installed.
//! As of revision 6674d785, the upstream release is suitable for that. Still, installation is a
//! bit cumbersome as it requires a particular nightly version:
//!
//!     $ git clone https://github.com/immunant/c2rust/
//!     $ cd c2rust
//!     $ rustup install nightly-2019-12-05
//!     $ rustup component add --toolchain nightly-2019-12-05 rustfmt rustc-dev
//!     $ cargo +nightly-2019-12-05 install --locked --debug --path c2rust
//!
//! [C2Rust]: https://c2rust.com/
//!
//! Usually, the `c2rust` binary is selected through the `PATH` environment variable.
//! The `C2RUST` environment variable can be used to override this.
//!
//! ## Versioning
//!
//! `riot-sys` is versioned using SemVer,
//! and efforts are made to not make breaking changes even while in the 0.x phase.
//! Note that as it passes on RIOT internals,
//! any of the SemVer guarantees only hold when built on the *same* RIOT --
//! once the underlying C code is changed, all bets are off.
//! Users of `riot-rs` can introspect its markers (see `build.rs`)
//! to influence which symbols to use.
//!
//! ---
//!
//! The types and constants of RIOT are translated in two forms:
//! through bindgen (to be linked together), and through C2Rust (transpiled, to be inlined).
//! This is necessary because neither can express the full set of useful RIOT APIs.
//!
//! All bindgen types are reexported in the main module and exclusively public through there. The
//! C2Rust types largely reside in the [inline] module, with some pub used into the root module as
//! necessary or convenient.
#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(feature = "keep-extern-types", feature(extern_types))]

#[deprecated(note = "Use core::ffi types directly")]
pub mod libc;

mod intrinsics_replacements;

mod bindgen;
pub mod inline;

include!(concat!(env!("OUT_DIR"), "/toplevel_from_inline.rs"));
pub use bindgen::*;

// re-export RIOT-rs core (used by riot-wrappers)
#[cfg(feature = "riot-rs")]
pub use riot_rs_core;
