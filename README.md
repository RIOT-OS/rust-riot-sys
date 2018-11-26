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

Currently, that information is transported in two components: A preprocessed
header file that has all includes and ifdefs expanded, and the CFLAGS
themselves. (It should be possible to later only use the CFLAGS).

To generate the expanded header file, you can insert a snippet like this into
your RIOT Makefile:

~~~~

for-cargo: ${BINDIR}/riot-expanded-headers.h

${BINDIR}/riot-expanded-headers.h: ../../riot-sys/riot-all.h
	$(Q)$(CC) $(CFLAGS_WITH_MACROS) $(INCLUDES) $< -fdirectives-only -E -o $@
	@echo "You may now run cargo with RIOT_EXPANDED_HEADER=$@ RIOT_CFLAGS=\"${RIOT_CFLAGS}\""

.PHONY: for-cargo
~~~~

Then, run `make for-cargo` (possibly with your `BOARD=` parameter), and prefix
the resulting environment variables to your cargo invocations. This crate's
build.rs script will take them up, adopt the relevant C flags, and populate the
`riot_sys` crate with the raw bindings.

Please note that riot-sys does not make any attempt to alter the Cargo target;
it typically needs to be set to `thumbv7m-none-eabi` for ARM Cortex devices, or
to `i686-unknown-linux-gnu` for the native board.

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
