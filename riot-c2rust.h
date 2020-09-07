// FIXME: This loses the atomic properties of inlined code, which is obviously bad.
//
// Proper fix: resolve https://github.com/immunant/c2rust/issues/293
//
// Until that's done, used inline functions need to be checked manually for
// whether they do atomic stuff.
#define _Atomic(x) x

#include <shell.h>
#include <thread.h>
#include <irq.h>
#include <stdio_base.h>
#include <periph/adc.h>
#include <periph/gpio.h>
#include <periph/i2c.h>
#include <net/gnrc.h> // needs atomic hack
#include <net/gnrc/udp.h> // needs atomic hack
#include <net/gnrc/pktbuf.h>
#include <net/gnrc/ipv6.h> // needs atomic hack
#include <net/gnrc/nettype.h>
#include <net/gnrc/netapi.h>

#include <saul.h>
#include <saul_reg.h>

#include <board.h>
#include <xtimer.h> // needs atomic hack

// not in riot-all?
#include <mutex.h>
