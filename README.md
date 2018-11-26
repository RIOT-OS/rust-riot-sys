This crate contains dynamically generated Rust FFI bindings to the [RIOT
Operating System](https://riot-os.org/).

Those bindings are inherently unsafe; it is recommended that their safe
abstractions in the riot-wrappers crate are used in most applications.

RIOT integration
----------------

Both the presence of API components and the contents of structs depend on
configuration set in the RIOT build system, eg. the presence of features or the
CPU used. This does not only affect the preprocessed C code, but also compiler
flags that govern the effective sizes of structs and need to be known to Cargo.

All the relevant information -- including the location of the actually used
RIOT header files -- is contained in the RIOT environment variables
`CFLAGS_WITH_MACROS` and `INCLUDES`; both need to be passed in to the Rust
build system as a `RIOT_CFLAGS` environment variable.

When using riot-sys, it is usually easiest to run from a target within the Make
system like this:

~~~~
target/thumbv7m-none-eabi/debug/libmy_app.a: always
	CC= CFLAGS= CPPFLAGS= RIOT_FLAGS="$(CFLAGS_WITH_MACROS) $(INCLUDES)" cargo build --target thumbv7m-none-eabi

.PHONY: always
~~~~

(CFLAGS etc. need to be cleared, for otherwise Cargo would assume those are
host flags.)

Extension
---------

Currently, only a subset of all the RIOT headers is processed; all the relevant
header files are included in this crate's `riot-all.h` header file. If you need
access to more RIOT APIs, more includes can be added there.

License
-------

This crate is licensed under the same terms as of the LGPL 2.1, following the
license terms of the RIOT Operating System.

It is maintained by Christian M. Amsüss <ca@etonomy.org> as part of the etonomy
project, see <https://etonomy.org/>.
